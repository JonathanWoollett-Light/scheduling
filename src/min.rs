use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use crate::*;

// O(m*n^3 * log(m*n) * min(m,n))
// O(m n^3)
pub fn schedule<T: Distance + Clone + Copy>(
    mut agents: Vec<Agent<T>>,
    mut tasks: HashMap<usize, Task<T>>,
) -> (Vec<(usize, usize)>, f32) {
    // Gets distances from all agents to all tasks
    // distances[n] represents the distances from the states of all agents to the start state of task n

    let mut assignments: Vec<(usize, usize)> = Vec::with_capacity(tasks.len());
    let mut distances: Vec<f32> = vec![0f32; agents.len()];

    // O(n/m)
    while !tasks.is_empty() {
        // O(m*n)
        let mut inner_distances: Vec<(usize, usize, f32)> = agents
            .iter()
            .enumerate()
            .flat_map(|(ai, a)| {
                tasks
                    .iter()
                    .map(|(ti, t)| (ai, *ti, a.state.distance(&t.from)))
                    .collect::<Vec<(usize, usize, f32)>>()
            })
            .collect();

        // O(m*n*log(m*n))
        inner_distances.sort_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap());

        // O(min(m,n))
        let mut inner_assignments: HashMap<usize, usize> = HashMap::with_capacity(tasks.len());
        let mut i = 0usize;
        for (ai, ti, d) in inner_distances.iter() {
            if let Ok(_) = insert_pair(&mut inner_assignments, (*ai, *ti)) {
                // Update distances
                let task = tasks.get(ti).unwrap();
                distances[*ai] += d + task.from.distance(&task.to);
                agents[*ai].state = task.to;

                i += 1;
                tasks.remove(ti);
                if i == tasks.len() {
                    break;
                }
            }
        }
        assignments.extend(inner_assignments.into_iter());
    }
    // O(m)
    (
        assignments,
        distances
            .into_iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
    )
}

// TODO This could be better, requires multiple mutable borrows and way to tell borrow checker they don't intersect
fn insert_pair<T: Hash + Eq + Clone>(map: &mut HashMap<T, T>, pair: (T, T)) -> Result<(), ()> {
    match map.entry(pair.0.clone()) {
        Entry::Occupied(_) => return Err(()),
        Entry::Vacant(e) => e.insert(pair.1.clone()),
    };
    match map.entry(pair.1) {
        Entry::Occupied(_) => {
            map.remove(&pair.0);
            return Err(());
        }
        Entry::Vacant(e) => e.insert(pair.0),
    };
    Ok(())
}

pub fn max(m: u128, mut n: u128) -> u128 {
    let mut num = 0;
    while n != 0 {
        let min = std::cmp::min(m, n);
        num += (m * n) + (m * n * (((m * n) as f32).log2() as u128)) + (min);
        n -= min;
    }
    return num + m;
}
