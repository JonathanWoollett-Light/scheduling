use num_format::{Locale, ToFormattedString};
use rand::distributions::{Distribution, Uniform};
use std::{sync::atomic::AtomicUsize, time::Instant, usize};

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
struct Root<T: Distance> {
    children: Vec<Node<T>>,
    min_path_time: f32,
}

// (size by size) world
// a larger size allows more obvious choices (better restriction) and thus less edges
const SIZE: usize = 1000;
const NUM_OF_TASKS: u128 = 8; // n
const NUM_OF_AGENTS: u128 = 5; // m
const PRINT_PATH: bool = false;

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
    // Some(|d|d>SIZE as f32)
    let (path, root) = dfs(
        &agents,
        &tasks,
        Some(|d| d > 2.5f32 * SIZE as f32 / NUM_OF_AGENTS as f32),
        false,
    );
    println!("time:\t{}", time(now));

    //let nodes: usize = tree.iter().map(|t| count_nodes(t)).sum();
    let nodes = root_count_edges(&root);
    println!("edges:");
    let max = max_nodes(NUM_OF_AGENTS, NUM_OF_TASKS);
    println!("\tmax:\t{: >15}", max.to_formatted_string(&Locale::en));
    println!("\tactual:\t{: >15}", nodes.to_formatted_string(&Locale::en));
    println!("\tinline:\t{:.?}", EDGE_COUNTER);
    println!("\t%:\t{:.6?}", 100f32 * (nodes as f32 / max as f32));

    //println!("{:#?}", tree);
    println!("min_time: {:.2}", root.min_path_time);
    if PRINT_PATH {
        println!("path: \n{:#?}", path);
    }
}

// TODO There has got to be a better way to do this (I'm guessing `impl Iterator for Root` and `impl Iterator for Node`)
fn root_count_edges<T: Distance>(root: &Root<T>) -> usize {
    let mut count = 0;
    for child in root.children.iter() {
        count += node_count_edges(child);
    }
    return count;

    fn node_count_edges<T: Distance>(node: &Node<T>) -> usize {
        let mut count = 1;
        for child in node.children.iter() {
            count += node_count_edges(child);
        }
        count
    }
}

fn max_nodes(m: u128, n: u128) -> u128 {
    (0u128..n)
        .map(|i| (0..i + 1).map(|j| m * (n - j)).product::<u128>())
        .sum::<u128>()
}

static EDGE_COUNTER: AtomicUsize = AtomicUsize::new(0);
fn dfs<T: Distance + Clone + Copy>(
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

            EDGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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

                EDGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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

fn time(instant: Instant) -> String {
    let mut millis = instant.elapsed().as_millis();
    let seconds = (millis as f32 / 1000f32).floor();
    millis %= 1000;
    let time = format!("{:#02}:{:#03}", seconds, millis);
    time
}
