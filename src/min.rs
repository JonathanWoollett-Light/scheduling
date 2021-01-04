fn min_assignment<T:Distance+Clone+Copy>(
    agents: &[Agent<T>],
    tasks: &[Task<T>]
) /*-> Vec<Edge<T>>*/ {
    // Gets distances from all agents to all tasks
    // distances[n] represents the distances from the states of all agents to the start state of task n
    let mut distances:Vec<(usize,usize,f32)> = agents.iter().enumerate().flat_map(
        |(ai,a)|
        tasks.iter().enumerate().map(
            |(ti,t)|
            (ai,ti,a.state.distance(&t.from))
        ).collect::<Vec<(usize,usize,f32)>>()
    ).collect();
    distances.sort_by(|(_,_,a),(_,_,b)|a.partial_cmp(b).unwrap());
    let assignments:Vec<Edge<T>> = Vec::with_capacity(tasks.len());
}