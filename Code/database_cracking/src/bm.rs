extern crate cracking;
extern crate rand;
extern crate time;

use cracking::db::*;
use std::collections::HashMap;
use std::iter;
use rand::Rng;
use time::PreciseTime;
use time::SteadyTime;

fn main() {
    benchmark_sparse_bfs_csv(
        vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000,
             10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000,
             10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000]);
}

// Given a list of numbers, does a bfs benchmark for sparse graphs with a number of nodes given
// by each value of the list.
// This function prints to stdout a valid csv file containing the results.
fn benchmark_sparse_bfs_csv(ns: Vec<i64>) {
    println!("nodes,edges,density,unoptimised,adaptive,preclustered");
    for n in ns {
        benchmark_sparse_bfs(n);
    }
}

// Given a number of nodes N, produces a sparse connected graph of that many nodes and gets runtime
// performance for each of adaptive, unoptimised and preclustering methods. It prints to stdout a
// line of a csv file.
fn benchmark_sparse_bfs(n: i64) {
    let adjacency_list = randomly_connected_tree(n);
    let all_nodes: Vec<i64> = (1..(n+1)).map(|x|x as i64).collect();
    let start_node = *rand::thread_rng().choose(&all_nodes).unwrap();
    print!("{},{},{}", n, adjacency_list.count, graph_density(n, adjacency_list.count));
    time_bfs(unoptimised_bfs,  &mut adjacency_list.clone(), start_node);
    time_bfs(adaptive_bfs,     &mut adjacency_list.clone(), start_node);
    time_bfs(preclustered_bfs, &mut adjacency_list.clone(), start_node);
    println!();
}

// Times a given bfs function against a given adjacency list using a given start node.
fn time_bfs<F>(mut bfs: F, mut adjacency_list: &mut Table, start_node: i64) where F: FnMut(&mut Table, i64) -> Vec<i64> {
    let start = PreciseTime::now();
    let visited = bfs(&mut adjacency_list, start_node);
    let end = PreciseTime::now();
    print!(",{}", start.to(end));
}

// Finds the directed density of a graph with n nodes and e edges. Returned as a float.
fn graph_density(n: i64, e: usize) -> f64 {
    (e as f64) / ((n * (n - 1)) as f64)
}

/* GRAPH BUILDING:
    Given a number of nodes N for a graph

    Returns an adjacency list (Table) for a connected graph with N nodes.
*/

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

// Returns a connected graph for n nodes, which are numbered 1-n inclusive.
fn randomly_connected_tree(n: i64) -> Table {
    let mut t = Table::new();
    t.new_columns(vec!["src".to_string(), "dst".to_string()]);
    let mut add_order: Vec<i64> = deal(n as usize).iter().map(|x|1 + *x as i64).collect();

    let node_1 = *rand::thread_rng().choose(&add_order).unwrap();
    let mut node_2 = *rand::thread_rng().choose(&add_order).unwrap();
    while node_2 == node_1 {
        node_2 = *rand::thread_rng().choose(&add_order).unwrap();
    }

    let index_1 = add_order.iter().position(|node| *node == node_1).unwrap();
    add_order.remove(index_1);
    let index_2 = add_order.iter().position(|node| *node == node_2).unwrap();
    add_order.remove(index_2);

    let mut src_col = vec![node_1, node_2];
    let mut dst_col = vec![node_2, node_1];
    for src in add_order {
        let dst = *rand::thread_rng().choose(&src_col).unwrap();

        src_col.push(src);
        dst_col.push(dst);

        src_col.push(dst);
        dst_col.push(src);
    }
    let mut connections = HashMap::new();
    connections.insert("src".to_string(), src_col);
    connections.insert("dst".to_string(), dst_col);
    t.insert(&mut connections);
    t.set_crk_col("src".to_string());
    let t_count = t.count;
    t.rearrange(deal(t_count).iter());
    t
}

/* BFS:
    Given a 2-column ADJACENCY_LIST of src_node!dst_node, this function visits every node in the
    graph from START_NODE.

    Returns the nodes visited in the order in which they were visited.
*/

fn discover(dst: i64, visited: &mut Vec<i64>, frontier: &mut Vec<i64>) {
    if !visited.contains(&dst) && !frontier.contains(&dst) {
        frontier.push(dst);
    }
}

macro_rules! t {
    ($work:expr, $tvar:ident) => {
            let start = PreciseTime::now();
            $work;
            let end = PreciseTime::now();
            $tvar = $tvar + start.to(end);
        };
}

fn adaptive_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let mut frontier = vec![start_node];
    let mut visited  = Vec::new();

    while !frontier.is_empty() {
        // Add visited nodes
        visited.append(&mut frontier.clone());

        let prev_frontier = frontier.clone();
        frontier.clear();
        // For each src in the previous frontier, find the dsts which haven't been visited yet,
        // and add them to a new, empty frontier.
        for src in prev_frontier {
            let selection = adjacency_list.cracker_select_in_three(src, src, true, true);
            let neighbours = (*(selection.get_col("dst".to_string()).unwrap())).v.clone();
            for dst in neighbours {
                discover(dst, &mut visited, &mut frontier);
            }
        }
    }
    visited
}

fn unoptimised_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let src_col = adjacency_list.get_col("src".to_string()).unwrap().v.clone();
    let dst_col = adjacency_list.get_col("dst".to_string()).unwrap().v.clone();
    let mut frontier = vec![start_node];
    let mut visited  = Vec::new();
    while !frontier.is_empty() {
        visited.append(&mut frontier.clone());
        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            for i in 0..src_col.len() {
                if src_col[i] == src {
                    discover(dst_col[i], &mut visited, &mut frontier);
                }
            }
        }
    }
    visited
}

fn preclustered_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let mut src_col = adjacency_list.get_col("src".to_string()).unwrap().v.clone();
    let mut dst_col = adjacency_list.get_col("dst".to_string()).unwrap().v.clone();
    let mut row_store: Vec<(i64, i64)> = Vec::with_capacity(adjacency_list.count);
    for i in 0..adjacency_list.count {
        row_store.push((src_col[i], dst_col[i]));
    }
    row_store.sort_by_key(|&k|k.0);
    for i in 0..adjacency_list.count {
        src_col[i] = row_store[i].0;
        dst_col[i] = row_store[i].1;
    }

    let mut frontier = vec![start_node];
    let mut visited  = Vec::new();
    while !frontier.is_empty() {
        visited.append(&mut frontier.clone());
        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            match src_col.binary_search(&src) {
                Ok(i) => {
                    let mut inc_idx = i.clone();
                    let mut dec_idx = i.clone();
                    loop {
                        discover(dst_col[inc_idx], &mut visited, &mut frontier);
                        inc_idx += 1;
                        if inc_idx >= src_col.len() {
                            break;
                        } else if src_col[inc_idx] != src {
                            break;
                        }
                    }
                    while src_col[dec_idx] == src {
                        discover(dst_col[dec_idx], &mut visited, &mut frontier);
                        if dec_idx == 0 {
                            break;
                        } else {
                            dec_idx -= 1;
                        }
                    }
                },
                Err(_e) => panic!("preclustered_bfs: Binary search on a node that doesn't exist"),
            }
        }
    }
    visited
}