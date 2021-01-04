use rand::distributions::Distribution;
use std::time::Instant;

use crate::core::*;

pub fn time(instant: Instant) -> String {
    let mut millis = instant.elapsed().as_millis();
    let seconds = (millis as f32 / 1000f32).floor();
    millis %= 1000;
    let time = format!("{:#02}:{:#03}", seconds, millis);
    time
}

#[derive(Debug, Copy, Clone)]
pub struct Coord {
    x: usize,
    y: usize,
}
impl Coord {
    pub fn new(
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

// TODO Make this generic
pub fn gen(
    dist: &rand::distributions::Uniform<usize>,
    mut rng: &mut rand::rngs::ThreadRng,
    num_of_tasks: u16,
    num_of_agents: u16,
) -> (Vec<Agent<Coord>>, Vec<Task<Coord>>) {
    let agents: Vec<Agent<Coord>> = (0..num_of_agents)
        .map(|_| Agent {
            state: Coord::new(&dist, &mut rng),
        })
        .collect();
    //println!("agents: {:?}", agents);

    let tasks: Vec<Task<Coord>> = (0..num_of_tasks)
        .map(|i| Task {
            id: i as usize,
            from: Coord::new(&dist, &mut rng),
            to: Coord::new(&dist, &mut rng),
        })
        .collect();

    (agents, tasks)
}
