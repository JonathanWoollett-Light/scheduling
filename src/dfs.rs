use crate::core::*;

pub trait Children {
    fn children(&self) -> u128;
}

#[derive(Debug, Copy, Clone)]
pub struct Edge<T: Distance> {
    agent: usize,
    task: usize,
    path: Option<(T, T, T, f32)>,
}
#[derive(Debug)]
pub struct Node<T: Distance> {
    edge: Edge<T>, // edge leading to this node
    children: Vec<Box<Node<T>>>,
    min_path_time: f32,
}
impl<T: Distance> Children for Node<T> {
    fn children(&self) -> u128 {
        1u128 + self.children.iter().map(|c| c.children()).sum::<u128>()
    }
}
pub struct Root<T: Distance> {
    children: Vec<Node<T>>,
    pub min_path_time: f32,
}
impl<T: Distance> Children for Root<T> {
    fn children(&self) -> u128 {
        self.children.iter().map(|c| c.children()).sum()
    }
}

pub fn search<T: Distance + Clone + Copy>(
    agents: &[Agent<T>],
    tasks: &[Task<T>],
    restriction: Option<fn(f32) -> bool>,
    path_checking: bool,
) -> (Vec<Edge<T>>, Root<T>) {
    let mut roots: Vec<Node<T>> = Vec::new();
    for (ti, agent) in agents.iter().enumerate() {
        for (ji, task) in tasks.iter().enumerate() {
            let to_task_distance = agent.state.distance(&task.from);
            let task_distance = task.from.distance(&task.to);
            let distance = to_task_distance + task_distance;

            let mut new_times = vec![0f32; agents.len()];
            new_times[ti] = distance;
            let mut new_agents = agents.to_vec();
            new_agents[ti].state = task.to;

            if let Some(res_fn) = restriction {
                if res_fn(to_task_distance) {
                    continue;
                }
            }

            roots.push(branch(
                new_agents,
                (0..tasks.len())
                    .filter_map(|i| if i == ji { None } else { Some(&tasks[i]) })
                    .collect::<Vec<&Task<T>>>(),
                restriction,
                Edge {
                    agent: ti,
                    task: task.id,
                    path: if path_checking {
                        Some((agent.state, task.from, task.to, distance))
                    } else {
                        None
                    },
                },
                new_times,
                path_checking,
            ));
        }
    }
    //println!("{:#?}", roots);
    let (time, path) = traverse_fastest(&roots);
    return (
        path,
        Root {
            children: roots,
            min_path_time: time,
        },
    );
    // return (time, path, roots);

    fn branch<T: Distance + Clone + Copy>(
        agents: Vec<Agent<T>>,
        tasks: Vec<&Task<T>>,
        restriction: Option<fn(f32) -> bool>,
        edge: Edge<T>,
        times: Vec<f32>,
        path_checking: bool,
    ) -> Node<T> {
        let mut path = Node {
            edge,
            children: Vec::new(),
            min_path_time: f32::MAX,
        };
        if tasks.is_empty() {
            path.min_path_time = *times
                .iter()
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap()
        }
        for (ti, agent) in agents.iter().enumerate() {
            for (ji, task) in tasks.iter().enumerate() {
                let to_task_distance = agent.state.distance(&task.from);
                let task_distance = task.from.distance(&task.to);
                let distance = to_task_distance + task_distance;

                let mut new_times = times.clone();
                new_times[ti] += distance;
                let mut new_agents = agents.clone();
                new_agents[ti].state = task.to;

                if let Some(res_fn) = restriction {
                    if res_fn(to_task_distance) {
                        continue;
                    }
                }

                let child = branch(
                    new_agents,
                    (0..tasks.len())
                        .filter_map(|i| if i == ji { None } else { Some(tasks[i]) })
                        .collect::<Vec<&Task<T>>>(),
                    restriction,
                    Edge {
                        agent: ti,
                        task: task.id,
                        path: if path_checking {
                            Some((agent.state, task.from, task.to, distance))
                        } else {
                            None
                        },
                    },
                    new_times,
                    path_checking,
                );

                if child.min_path_time < path.min_path_time {
                    path.min_path_time = child.min_path_time;
                }

                path.children.push(Box::new(child));
            }
        }
        path
    }
    fn traverse_fastest<T: Distance + Copy>(tree: &[Node<T>]) -> (f32, Vec<Edge<T>>) {
        let min = tree
            .iter()
            .min_by(|x, y| (*x).min_path_time.partial_cmp(&(*y).min_path_time).unwrap())
            .unwrap();
        //println!("root min: {:.?}",min);
        let mut path: Vec<Edge<T>> = vec![min.edge];
        path.append(&mut traverse_val(&min.children, min.min_path_time));

        return (min.min_path_time, path);
        fn traverse_val<T: Distance + Copy>(tree: &[Box<Node<T>>], val: f32) -> Vec<Edge<T>> {
            let min = tree
                .iter()
                .find(|x| (*x).min_path_time.partial_cmp(&val) == Some(std::cmp::Ordering::Equal))
                .expect("Bad path");
            //println!("min: {:.?}",min);
            let mut path: Vec<Edge<T>> = vec![min.edge];
            if !min.children.is_empty() {
                path.append(&mut traverse_val(&min.children, min.min_path_time));
            }
            path
        }
    }
}

pub fn max_nodes(m: u128, n: u128) -> u128 {
    (0u128..n)
        .map(|i| (0..i + 1).map(|j| m * (n - j)).product::<u128>())
        .sum::<u128>()
}
