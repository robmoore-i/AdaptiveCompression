extern crate cracking;
extern crate rand;

use cracking::db::*;
use std::collections::HashMap;
use std::iter;

fn main() {
    let n = 5;
    let mut t = fully_connected_graph(n);
    let selection = t.select_in_two("src".to_string(), n+1);
    match selection.get_col("src".to_string()) {
        Some(ref col) => println!("src = {:?}", col.v),
        None          => println!("src failed"),
    }
    match selection.get_col("dst".to_string()) {
        Some(ref col) => println!("dst = {:?}", col.v),
        None          => println!("dst failed"),
    }
    bfs(&mut t, 3);
}

// Deals out the numbers from 0 to n-1 inclusive in a random order as usizes.
fn deal(n: usize) -> Vec<usize> {
    let mut dealing: Vec<i64> = Vec::with_capacity(n as usize);
    let half_n_as_f64 = (0.5*(n%2) as f64)+(n / 2) as f64;
    let scaling_factor: f64 = half_n_as_f64 / (<i64>::max_value() as f64);
    while dealing.len() < n {
        let candidate: f64 = (half_n_as_f64 + scaling_factor * (rand::random::<i64>() as f64)).floor();
        let candidate_as_i64 = candidate as i64;
        if !dealing.contains(&candidate_as_i64) {
            dealing.push(candidate_as_i64);
        }
    }
    let mut dealing_usize = Vec::with_capacity(n as usize);
    for x in dealing.iter_mut() {
        dealing_usize.push(*x as usize);
    }
    dealing_usize
}

// A randomly shuffled adjacency list representing a complete graph for n nodes, columns are src, dst,
// and src is the cracked column.
fn fully_connected_graph(n: i64) -> Table {
    let mut t = Table::new();
    t.new_columns(vec!["src".to_string(), "dst".to_string()]);
    for i in 1..(n+1) {
        let mut connections = HashMap::new();
        let i_vec: Vec<i64> = iter::repeat(i as i64).take((n-1) as usize).collect();
        connections.insert("src".to_string(), i_vec);
        let mut dst_nodes: Vec<i64> = Vec::with_capacity((n-1) as usize);
        dst_nodes.extend(1..i);
        dst_nodes.extend((i+1)..(n+1));
        connections.insert("dst".to_string(), dst_nodes);
        t.insert(&mut connections)
    }
    t.set_crk_col("src".to_string());
    let t_count = t.count;
    t.rearrange(deal(t_count).iter());
    t
}

// Given a 2-column ADJACENCY_LIST of src_node!dst_node, this function
// visits every node in the graph from START_NODE.
fn bfs(adjacency_list: &mut Table, start_node: i64) {
    let mut frontier = vec![start_node];
    let mut visited = Vec::new();
    while !frontier.is_empty() {
        println!("Visited {:?}", frontier);
        // Add visited nodes
        visited.append(&mut frontier.clone());
        let prev_frontier = frontier.clone();
        frontier.clear();
        // For each src in the previous frontier, find the dsts which haven't been visited yet,
        // and add them to a new, empty frontier.
        for src in prev_frontier {
            match adjacency_list.cracker_select_in_three(src - 1, src + 1, false, false).get_col("dst".to_string()) {
                Some(ref col) => for dst in col.v.clone() {
                    if !visited.contains(&dst) {
                        frontier.push(dst);
                    }
                },
                None => panic!("bfs: No dst column in crack_in_three result for src node {}", src),
            }
        }
    }
}