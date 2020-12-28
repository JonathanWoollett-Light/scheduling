use num_format::{Locale, ToFormattedString};
use rand::distributions::{Distribution, Uniform};
use std::{time::Instant,sync::atomic::{AtomicUsize,Ordering}};

#[derive(Debug, Copy, Clone)]
struct Coord {
    x: usize,
    y: usize,
}
#[derive(Debug)]
struct Task {
    id: usize,
    from: Coord,
    to: Coord,
}
impl Task {
    fn distance(&self) -> f32 {
        distance(&self.from, &self.to)
    }
}

//fn distance(from: &Coord, to: &Coord) -> f32 {
//     (((from.x as f32 - to.x as f32).powf(2f32) + (from.y as f32 - to.y as f32).powf(2f32))).sqrt()
// }
fn distance(from: &Coord, to: &Coord) -> f32 {
    (from.x as f32 - to.x as f32).abs() + (from.y as f32 - to.y as f32).abs()
}

#[derive(Debug, Copy, Clone)]
struct Edge {
    agent: usize,
    task: usize,
    path: Option<(Coord, Coord, Coord, f32)>,
}

#[derive(Debug)]
struct Node {
    edge: Edge, // edge leading to this node
    children: Vec<Box<Node>>,
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

    let agents: Vec<Coord> = (0..NUM_OF_AGENTS)
        .map(|_| Coord {
            x: dist.sample(&mut rng),
            y: dist.sample(&mut rng),
        })
        .collect();
    //println!("agents: {:?}", agents);

    let tasks: Vec<Task> = (0..NUM_OF_TASKS)
        .map(|i| Task {
            id: i as usize,
            from: Coord {
                x: dist.sample(&mut rng),
                y: dist.sample(&mut rng),
            },
            to: Coord {
                x: dist.sample(&mut rng),
                y: dist.sample(&mut rng),
            },
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
    println!("{:.?}",EDGE_COUNTER);
}

fn count_nodes(node: &Box<Node>) -> usize {
    let mut count = 1;
    for child in (*node.children).iter() {
        count += count_nodes(child);
    }
    return count;
}
static EDGE_COUNTER: AtomicUsize = AtomicUsize::new(0);
fn dfs(
    agents: &Vec<Coord>,
    tasks: &Vec<Task>,
    path_checking: bool,
) -> (f32, Vec<Edge>, Vec<Box<Node>>) {
    let mut roots: Vec<Box<Node>> = Vec::new();
    for (ti, taxi) in agents.iter().enumerate() {
        for (ji, journey) in tasks.iter().enumerate() {
            let distance = distance(taxi, &journey.from) + journey.distance();
            let mut new_times = vec![0f32; agents.len()];
            new_times[ti] = distance;
            let mut new_agents = agents.clone();
            new_agents[ti] = journey.to;

            EDGE_COUNTER.fetch_add(1,Ordering::SeqCst);
            roots.push(Box::new(branch(
                new_agents,
                (0..tasks.len())
                    .filter_map(|i| if i == ji { None } else { Some(&tasks[i]) })
                    .collect::<Vec<&Task>>(),
                Edge {
                    agent: ti,
                    task: journey.id,
                    path: if path_checking {
                        Some((*taxi, journey.from, journey.to, distance))
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

    fn branch(
        agents: Vec<Coord>,
        tasks: Vec<&Task>,
        edge: Edge,
        times: Vec<f32>,
        path_checking: bool,
    ) -> Node {
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
        for (ti, taxi) in agents.iter().enumerate() {
            for (ji, journey) in tasks.iter().enumerate() {
                let distance = distance(taxi, &journey.from) + distance(&journey.from, &journey.to);
                let mut new_times = times.clone();
                new_times[ti] += distance;
                let mut new_agents = agents.clone();
                new_agents[ti] = journey.to;

                EDGE_COUNTER.fetch_add(1,Ordering::SeqCst);
                let child = branch(
                    new_agents,
                    (0..tasks.len())
                        .filter_map(|i| if i == ji { None } else { Some(tasks[i]) })
                        .collect::<Vec<&Task>>(),
                    Edge {
                        agent: ti,
                        task: journey.id,
                        path: if path_checking {
                            Some((*taxi, journey.from, journey.to, distance))
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
    fn traverse_fastest(tree: &Vec<Box<Node>>) -> (f32, Vec<Edge>) {
        let min = tree
            .iter()
            .min_by(|x, y| (*x).min_path_time.partial_cmp(&(*y).min_path_time).unwrap())
            .unwrap();
        //println!("root min: {:.?}",min);
        let mut path: Vec<Edge> = vec![min.edge];
        path.append(&mut traverse_val(&min.children, min.min_path_time));

        return (min.min_path_time, path);
        fn traverse_val(tree: &Vec<Box<Node>>, val: f32) -> Vec<Edge> {
            let min = tree
                .iter()
                .find(|x| (*x).min_path_time == val)
                .expect("Bad path");
            //println!("min: {:.?}",min);
            let mut path: Vec<Edge> = vec![min.edge];
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
