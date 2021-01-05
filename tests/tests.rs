// cargo test --release -- --nocapture

#[cfg(test)]
mod tests {
    use num_format::{Locale, ToFormattedString};
    use rand::distributions::Uniform;
    use scheduling::*;
    use std::{collections::HashMap, time::Instant, usize};

    use scheduling::dfs::Children;

    const SIZE: usize = 1000;
    const NUM_OF_TASKS: u16 = 7; // n
    const NUM_OF_AGENTS: u16 = 3; // m
    const PRINT_PATH: bool = false;

    #[test]
    fn dfs() {
        let mut rng = rand::thread_rng();
        let dist = Uniform::from(0..SIZE);

        let (agents, tasks) = gen(&dist, &mut rng, NUM_OF_TASKS, NUM_OF_AGENTS);

        //println!("agents: {:?}", agents);
        //println!("tasks: {:?}", tasks);

        let now = Instant::now();
        // Some(|d|d>SIZE as f32)
        let (path, root) = dfs::schedule(
            &agents,
            tasks
                .into_iter()
                .enumerate()
                .map(|(ti, t)| (ti, t))
                .collect(),
            Some(|d| d > 2.5f32 * SIZE as f32 / NUM_OF_AGENTS as f32),
            false,
        );
        println!("time:\t{}", time(now));

        let nodes = root.children();
        println!("edges:");
        let max = dfs::max(NUM_OF_AGENTS as u128, NUM_OF_TASKS as u128);
        println!("\tmax:\t{: >15}", max.to_formatted_string(&Locale::en));
        println!("\tactual:\t{: >15}", nodes.to_formatted_string(&Locale::en));
        println!("\t%:\t{:.5?}", 100f32 * (nodes as f32 / max as f32));

        println!("min_time: {:.2}", root.min_path_time);

        if PRINT_PATH {
            println!("path: \n{:#?}", path);
        }
    }
    #[test]
    fn min() {
        let mut rng = rand::thread_rng();
        let dist = Uniform::from(0..SIZE);

        let (agents, tasks) = gen(&dist, &mut rng, NUM_OF_TASKS, NUM_OF_AGENTS);

        //println!("agents: {:?}", agents);
        //println!("tasks: {:?}", tasks);

        let now = Instant::now();
        // Some(|d|d>SIZE as f32)
        let (path, min_time) = min::schedule(
            agents,
            tasks
                .into_iter()
                .enumerate()
                .map(|(ti, t)| (ti, t))
                .collect::<HashMap<_, _>>(),
        );
        println!("time:\t{}", time(now));

        let max = min::max(NUM_OF_AGENTS as u128, NUM_OF_TASKS as u128);
        println!("edges:\t{: >15}", max);

        println!("min_time: {:.2}", min_time);

        if PRINT_PATH {
            println!("path: \n{:#?}", path);
        }
    }
}
