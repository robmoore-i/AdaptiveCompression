extern crate cracking;
extern crate rand;
extern crate time;
extern crate bit_vec;

use cracking::db::*;
use std::collections::HashMap;
use std::iter;
use rand::Rng;
use time::PreciseTime;
use time::SteadyTime;
use bit_vec::BitVec;

fn main() {
//    let graph_sizes = graph_size_range(5, 5, 200, 5);
//    benchmark_sparse_bfs_csv(graph_sizes);

    let mut adjacency_list = Table::new();
    adjacency_list.new_columns(vec!["src".to_string(), "dst".to_string()]);
    let mut connections = HashMap::new();
    connections.insert("src".to_string(), vec![4, 2, 1, 4, 5, 4]);
    connections.insert("dst".to_string(), vec![1, 2, 3, 4, 5, 6]);
    adjacency_list.insert(&mut connections);
    adjacency_list.set_crk_col("src".to_string());

    let selected = adjacency_list.cracker_select_specific(4);
    println!("Selected {:?}", (*selected.get_col("src".to_string()).unwrap()).v);

    println!("Original {:?}", (*adjacency_list.get_col("src".to_string()).unwrap()).v);
    println!("Cracker  {:?}", adjacency_list.crk_col.crk);
    print!(  "Index    ");
    adjacency_list.crk_col.crk_idx.print_nodes(&mut vec![4, 5]);
    println!();
}

fn graph_size_range(n_readings: i64, min_graph_size: i64, max_graph_size: i64, step: i64) -> Vec<i64> {
    let mut bm_graph_sizes: Vec<i64> = Vec::new();
    let mut size = min_graph_size;
    for i in 0..n_readings {
        while size <= max_graph_size {
            bm_graph_sizes.push(size);
            size += (i * step / n_readings) + step;
        }
        size = min_graph_size;
    }
    bm_graph_sizes.push(max_graph_size);
    bm_graph_sizes
}

// Given a list of numbers, does a bfs benchmark for sparse graphs with a number of nodes given
// by each value of the list.
// This function prints to stdout a valid csv file containing the results.
fn benchmark_sparse_bfs_csv(graph_sizes: Vec<i64>) {
    println!("nodes,edges,density,unoptimised,adaptive,preclustered,preclusteredRLE");
    for n in graph_sizes {
        benchmark_sparse_bfs(n);
    }
}

// Given a number of nodes N, produces a sparse connected graph of that many nodes and gets runtime
// performance for each of adaptive, unoptimised and preclustering methods. It prints to stdout a
// line of a csv file.
fn benchmark_sparse_bfs(n: i64) {
    let adjacency_list = randomly_connected_tree(n);
    let all_nodes: Vec<i64> = (1..(n + 1)).map(|x| x as i64).collect();
    let start_node = *rand::thread_rng().choose(&all_nodes).unwrap();
    print!("{},{},{}", n, adjacency_list.count, graph_density(n, adjacency_list.count));
    time_bfs(unoptimised_bfs,      &mut adjacency_list.clone(), start_node);
    time_bfs(adaptive_bfs,         &mut adjacency_list.clone(), start_node);
    time_bfs(preclustered_bfs,     &mut adjacency_list.clone(), start_node);
    time_bfs(preclustered_rle_bfs, &mut adjacency_list.clone(), start_node);
    println!();
}

// Times a given bfs function against a given adjacency list using a given start node.
fn time_bfs<F>(mut bfs: F, mut adjacency_list: &mut Table, start_node: i64) where F: FnMut(&mut Table, i64) -> Vec<i64> {
    let start = PreciseTime::now();
    let visited = bfs(&mut adjacency_list, start_node);
    let end = PreciseTime::now();
    println!("time = {}, nodes_visited = {}", start.to(end), visited.len());
}

// Finds the directed density of a graph with n nodes and e edges. Returned as a float.
fn graph_density(n: i64, e: usize) -> f64 {
    (e as f64) / ((n * (n - 1)) as f64)
}

/* PROFILING
    Macros for profiling the performance of a function
*/

macro_rules! t_block {
    ($work:block, $tvar:ident) => {
            let start = PreciseTime::now();
            $work;
            let end = PreciseTime::now();
            $tvar = $tvar + start.to(end);
        };
}

macro_rules! t_expr {
    ($work:expr, $tvar:ident) => {
            let start = PreciseTime::now();
            $work;
            let end = PreciseTime::now();
            $tvar = $tvar + start.to(end);
        };
}

// Todo: Write a nice macro for printing all the profiler timing variables at the end of a fn.

/* GRAPH BUILDING:
    Given a number of nodes N for a graph

    Returns an adjacency list (Table) for a connected graph with N nodes.
*/

// Deals out the numbers from 0 to n-1 inclusive in a random order as usizes.
fn deal(n: usize) -> Vec<usize> {
    let mut dealing: Vec<i64> = Vec::with_capacity(n as usize);
    let half_n_as_f64 = (0.5 * (n % 2) as f64) + (n / 2) as f64;
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

// Returns a randomly shuffled adjacency list representing a complete graph for n nodes
fn fully_connected_graph(n: i64) -> Table {
    let mut t = Table::new();
    t.new_columns(vec!["src".to_string(), "dst".to_string()]);
    for i in 1..(n + 1) {
        let mut connections = HashMap::new();
        let i_vec: Vec<i64> = iter::repeat(i as i64).take((n - 1) as usize).collect();
        connections.insert("src".to_string(), i_vec);
        let mut dst_nodes: Vec<i64> = Vec::with_capacity((n - 1) as usize);
        dst_nodes.extend(1..i);
        dst_nodes.extend((i + 1)..(n + 1));
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
    let mut add_order: Vec<i64> = deal(n as usize).iter().map(|x| 1 + *x as i64).collect();

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
fn discover(dst: i64, visited: &mut BitVec, frontier: &mut Vec<i64>) {
    if !visited.get((dst as usize) - 1).unwrap_or(false) && !frontier.contains(&dst) {
        frontier.push(dst);
    }
}

fn set_indices(bv: &mut BitVec, indices: Vec<i64>) {
    let l = bv.len();
    for i in indices {
        let i_usize = i as usize;
        if i_usize >= l {
            bv.grow(1 + i_usize - l, false);
        }
        bv.set(i_usize, true);
    }
}

fn bv_where(bv: BitVec) -> Vec<i64> {
    let mut v = Vec::with_capacity(bv.len());
    for i in 0..bv.len() {
        if bv.get(i).unwrap() {
            v.push(1 + i as i64);
        }
    }
    v
}

fn indicise(v: Vec<i64>) -> Vec<i64> {
    v.iter().map(|x|x-1).collect()
}

fn adaptive_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    while !frontier.is_empty() {
        // Add visited nodes
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        // For each src in the previous frontier, find the dsts which haven't been visited yet,
        // and add them to a new, empty frontier.
        for src in prev_frontier {
            let selection = adjacency_list.cracker_select_specific(src);
            let neighbours = (*(selection.get_col("dst".to_string()).unwrap())).v.clone();
            for dst in neighbours {
                discover(dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}

fn unoptimised_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let src_col = adjacency_list.get_col("src".to_string()).unwrap().v.clone();
    let dst_col = adjacency_list.get_col("dst".to_string()).unwrap().v.clone();
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));
        
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
    bv_where(visited)
}

fn preclustered_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let mut src_col = adjacency_list.get_col("src".to_string()).unwrap().v.clone();
    let mut dst_col = adjacency_list.get_col("dst".to_string()).unwrap().v.clone();
    let mut row_store = Vec::with_capacity(adjacency_list.count);
    for i in 0..adjacency_list.count {
        row_store.push((src_col[i], dst_col[i]));
    }
    row_store.sort_by_key(|&k| k.0);
    for i in 0..adjacency_list.count {
        src_col[i] = row_store[i].0;
        dst_col[i] = row_store[i].1;
    }

    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let i = src_col.binary_search(&src).unwrap();
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
        }
    }
    bv_where(visited)
}

fn preclustered_rle_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let src_col = adjacency_list.get_col("src".to_string()).unwrap().v.clone();
    let dst_col = adjacency_list.get_col("dst".to_string()).unwrap().v.clone();

    let mut encoded_col: Vec<Vec<i64>> = Vec::new();
    for i in 0..adjacency_list.count {
        let src_as_usize = src_col[i] as usize;
        let dst = dst_col[i];

        while encoded_col.len() <= src_as_usize {
            encoded_col.push(Vec::new());
        }

        if encoded_col[src_as_usize].is_empty() {
            encoded_col[src_as_usize] = vec![dst];
        } else {
            encoded_col[src_as_usize].push(dst);
        }
    }

    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);
    
    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));
        
        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            for dst in &encoded_col[src as usize] {
                discover(*dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}
