extern crate cracking;
extern crate rand;
extern crate time;

use cracking::db::*;
use std::collections::HashMap;
use std::iter;
use rand::Rng;
use time::PreciseTime;

fn main() {
    bfs_random_adaptive(100);
}

fn bfs_random_adaptive(n: i64) {
    let mut adjacency_list = randomly_connected_graph(n);
    let all_nodes: Vec<i64> = (1..(n+1)).map(|x|x as i64).collect();
    let start_node = *rand::thread_rng().choose(&all_nodes).unwrap();
    let start = PreciseTime::now(); bfs(&mut adjacency_list, start_node);
    let end = PreciseTime::now();
    println!("[BFS ; randomly connected graph ; adaptive clustering] Edge count: {} ; Running time: {}", adjacency_list.count, start.to(end));
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

// Returns a randomly shuffled adjacency list representing a complete graph for n nodes,
// columns are src, dst. src is the cracked column.
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

// Returns a connected graph for n nodes, which are numbered 1-n inclusive
fn randomly_connected_graph(n: i64) -> Table {
    // Add a random edge and node in a loop until all the nodes are created
    // Continue to add edges until there are e of them
    let mut t = Table::new();
    t.new_columns(vec!["src".to_string(), "dst".to_string()]);
    let all_nodes: Vec<i64> = (1..(n+1)).map(|x|x as i64).collect();
    let mut nodes: Vec<i64> = Vec::with_capacity(n as usize);
    nodes.push(*rand::thread_rng().choose(&all_nodes).unwrap());
    while nodes.len() < n as usize {
        let rand_src = *rand::thread_rng().choose(&nodes).unwrap();
        let rand_dst = *rand::thread_rng().choose(&all_nodes).unwrap();
        if rand_src != rand_dst {
            if !nodes.contains(&rand_dst) {
                nodes.push(rand_dst);
            }
            if !nodes.contains(&rand_src) {
                nodes.push(rand_src);
            }
            let mut connections = HashMap::new();
            connections.insert("src".to_string(), vec![rand_src, rand_dst]);
            connections.insert("dst".to_string(), vec![rand_dst, rand_src]);
            t.insert(&mut connections);
        }
    }
    t.set_crk_col("src".to_string());
    let t_count = t.count;
    t.rearrange(deal(t_count).iter());
    t
}

// Given a 2-column ADJACENCY_LIST of src_node!dst_node, this function visits every node in the
// graph from START_NODE.
// Returns the nodes visited in the order in which they were visited.
fn bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let mut frontier = vec![start_node];
    let mut visited = Vec::new();
    while !frontier.is_empty() {
        // Add visited nodes
        visited.append(&mut frontier.clone());
        let prev_frontier = frontier.clone();
        frontier.clear();
        // For each src in the previous frontier, find the dsts which haven't been visited yet,
        // and add them to a new, empty frontier.
        for src in prev_frontier {
            match adjacency_list.cracker_select_in_three(src, src, true, true).get_col("dst".to_string()) {
                Some(ref col) => for dst in col.v.clone() {
                    if !visited.contains(&dst) && !frontier.contains(&dst) {
                        frontier.push(dst);
                    }
                },
                None => panic!("bfs: No dst column in crack_in_three result for src node {}", src),
            }
        }
    }
    visited
}