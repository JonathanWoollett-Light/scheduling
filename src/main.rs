use num_format::{Locale, ToFormattedString};
use rand::distributions::{Distribution, Uniform};
use std::{
    usize,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

#[derive(Debug, Copy, Clone)]
struct Coord {
    x: usize,
    y: usize,
}
impl Coord {
    fn new(
        dist: &rand::distributions::Uniform<usize>,
        mut rng: &mut rand::rngs::ThreadRng,
    ) -> Self {
        Coord {
            x: dist.sample(&mut rng),
            y: dist.sample(&mut rng),
        }
    }
}
impl Distance for Coord {
    fn distance(&self, other: &Self) -> f32 {
        (self.x as f32 - other.x as f32).abs() + (self.y as f32 - other.y as f32).abs()
    }
}

#[derive(Debug, Copy, Clone)]
struct Agent<T: Distance + Clone> {
    state: T,
}

#[derive(Debug)]
struct Task<T: Distance> {
    id: usize,
    from: T,
    to: T,
}

#[derive(Debug, Copy, Clone)]
struct Edge<T: Distance> {
    agent: usize,
    task: usize,
    path: Option<(T, T, T, f32)>,
}

#[derive(Debug)]
struct Node<T: Distance> {
    edge: Edge<T>, // edge leading to this node
    children: Vec<Box<Node<T>>>,
    min_path_time: f32,
}

const SIZE: usize = 5; // (size by size) world
const NUM_OF_TASKS: u32 = 6;
const NUM_OF_AGENTS: u32 = 3;

fn main() {
    println!("Hello, world!");

    //return;
    let mut rng = rand::thread_rng();
    let dist = Uniform::from(0..SIZE);

    let agents: Vec<Agent<Coord>> = (0..NUM_OF_AGENTS)
        .map(|_| Agent {
            state: Coord::new(&dist, &mut rng),
        })
        .collect();
    //println!("agents: {:?}", agents);

    let tasks: Vec<Task<Coord>> = (0..NUM_OF_TASKS)
        .map(|i| Task {
            id: i as usize,
            from: Coord::new(&dist, &mut rng),
            to: Coord::new(&dist, &mut rng),
        })
        .collect();
    //println!("tasks: {:?}", tasks);

    let now = Instant::now();
    let (min_time, path, tree) = dfs(&agents, &tasks, false);
    println!("{}", time(now));

    let nodes: usize = tree.iter().map(|t| count_nodes(t)).sum();
    println!("nodes: {}", nodes.to_formatted_string(&Locale::en));

    //println!("{:#?}", tree);
    println!("{:#?}", path);
    println!("{:.2}", min_time);
    println!("nodes: {:.?}", EDGE_COUNTER);
}

fn count_nodes<T: Distance>(node: &Box<Node<T>>) -> usize {
    let mut count = 1;
    for child in (*node.children).iter() {
        count += count_nodes(child);
    }
    return count;
}
static EDGE_COUNTER: AtomicUsize = AtomicUsize::new(0);
fn dfs<T: Distance + Clone + Copy>(
    agents: &Vec<Agent<T>>,
    tasks: &Vec<Task<T>>,
    path_checking: bool,
) -> (f32, Vec<Edge<T>>, Vec<Box<Node<T>>>) {
    let mut roots: Vec<Box<Node<T>>> = Vec::new();
    for (ti, agent) in agents.iter().enumerate() {
        for (ji, task) in tasks.iter().enumerate() {
            let distance = agent.state.distance(&task.from) + task.from.distance(&task.to);
            let mut new_times = vec![0f32; agents.len()];
            new_times[ti] = distance;
            let mut new_agents = agents.clone();
            new_agents[ti].state = task.to;

            EDGE_COUNTER.fetch_add(1, Ordering::SeqCst);
            roots.push(Box::new(branch(
                new_agents,
                (0..tasks.len())
                    .filter_map(|i| if i == ji { None } else { Some(&tasks[i]) })
                    .collect::<Vec<&Task<T>>>(),
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
            )));
        }
    }
    //println!("{:#?}", roots);
    let (time, path) = traverse_fastest(&roots);
    return (time, path, roots);

    fn branch<T: Distance + Clone + Copy>(
        agents: Vec<Agent<T>>,
        tasks: Vec<&Task<T>>,
        edge: Edge<T>,
        times: Vec<f32>,
        path_checking: bool,
    ) -> Node<T> {
        let mut path = Node {
            edge,
            children: Vec::new(),
            min_path_time: f32::MAX,
        };
        if tasks.len() == 0 {
            path.min_path_time = *times
                .iter()
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap()
        }
        for (ti, agent) in agents.iter().enumerate() {
            for (ji, task) in tasks.iter().enumerate() {
                let distance =
                    agent.state.distance(&task.from) + task.from.distance(&task.to);
                let mut new_times = times.clone();
                new_times[ti] += distance;
                let mut new_agents = agents.clone();
                new_agents[ti].state = task.to;

                EDGE_COUNTER.fetch_add(1, Ordering::SeqCst);
                let child = branch(
                    new_agents,
                    (0..tasks.len())
                        .filter_map(|i| if i == ji { None } else { Some(tasks[i]) })
                        .collect::<Vec<&Task<T>>>(),
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
        return path;
    }
    fn traverse_fastest<T: Distance + Copy>(tree: &Vec<Box<Node<T>>>) -> (f32, Vec<Edge<T>>) {
        let min = tree
            .iter()
            .min_by(|x, y| (*x).min_path_time.partial_cmp(&(*y).min_path_time).unwrap())
            .unwrap();
        //println!("root min: {:.?}",min);
        let mut path: Vec<Edge<T>> = vec![min.edge];
        path.append(&mut traverse_val(&min.children, min.min_path_time));

        return (min.min_path_time, path);
        fn traverse_val<T: Distance + Copy>(tree: &Vec<Box<Node<T>>>, val: f32) -> Vec<Edge<T>> {
            let min = tree
                .iter()
                .find(|x| (*x).min_path_time == val)
                .expect("Bad path");
            //println!("min: {:.?}",min);
            let mut path: Vec<Edge<T>> = vec![min.edge];
            if min.children.len() > 0 {
                path.append(&mut traverse_val(&min.children, min.min_path_time));
            }
            return path;
        }
    }
}

fn time(instant: Instant) -> String {
    let mut millis = instant.elapsed().as_millis();
    let seconds = (millis as f32 / 1000f32).floor();
    millis = millis % 1000;
    let time = format!("{:#02}:{:#03}", seconds, millis);
    return time;
}
