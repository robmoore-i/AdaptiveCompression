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

/* MACROS/helpers
    For profiling the performance of a function and making hashmaps.
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

// Macro for making hashmaps
// I credit this macro (map) to this bod:
// https://stackoverflow.com/a/27582993/3803302
macro_rules! map (
        { $($key:expr => $value:expr),+ } => {
            {
                let mut m = ::std::collections::HashMap::new();
                $(
                    m.insert($key, $value);
                )+
                m
            }
        };
    );

// Prints a float vec where each float is to 4dp.
fn pretty_println_f64vec(floats: &Vec<f64>) {
    print!("[");
    print!("{}", floats[0]);
    for i in 1..floats.len() {
        print!(", {:.4}", floats[i]);
    }
    print!("]\n");
}

// Todo: Write a nice macro for printing all the profiler timing variables at the end of a fn.

/* MAIN
    Top level function. Executed with `cargo run [--release]` from terminal.
*/

fn main() {
    pagerank_example_test(preclustered_pagerank);
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
    // Put 1 - n in a bag.
    let mut dealing: Vec<usize> = Vec::with_capacity(n as usize);
    for i in 0..n {
        dealing.push(i);
    }
    let mut dealt: Vec<usize> = Vec::with_capacity(n as usize);
    let mut rng = rand::thread_rng();
    let mut chosen = 0;
    while dealing.len() > 1 {
        let random = rng.gen_range(0, n - chosen);
        dealt.push(dealing[random]);
        dealing.remove(random);
        chosen = chosen + 1;
    }
    dealt.push(dealing[0]);
    dealt
}

// Returns a randomly shuffled adjacency list representing a complete graph for n nodes
fn fully_connected_graph(n: i64) -> Table {
    let mut t = Table::new();
    t.new_columns(map!{"src" => 'j', "dst" => 'j'});
    for i in 1..(n + 1) {
        let i_vec: Vec<i64> = iter::repeat(i as i64).take((n - 1) as usize).collect();
        let mut dst_nodes: Vec<i64> = Vec::with_capacity((n - 1) as usize);
        dst_nodes.extend(1..i);
        dst_nodes.extend((i + 1)..(n + 1));
        t.insert(&mut map!{"src" => i_vec, "dst" => dst_nodes})
    }
    t.set_crk_col("src");
    let t_count = t.count;
    t.rearrange(deal(t_count).iter());
    t
}

// Returns a bidirectionally connected tree for n nodes, which are numbered 1 to n inclusive.
fn randomly_connected_tree(n: i64) -> Table {
    let mut t = Table::new();
    t.new_columns(map!{"src" => 'j', "dst" => 'j'});
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
    t.insert(&mut map!{"src" => src_col, "dst" => dst_col});
    t.set_crk_col("src");
    let t_count = t.count;
    t.rearrange(deal(t_count).iter());
    t
}

// Returns a connected graph for n nodes, which are numbered 1 to n inclusive, having e edges, where
// e is presumed to be larger than n.
// Todo: Generate edges using cantor's pairing function for efficient dedup.
fn randomly_connected_graph(n: i64, e: i64) -> Table {
    if e < n { println!("e < n, aborting."); return Table::new(); };
    let mut t = Table::new();
    t.new_columns(map!{"src" => 'j', "dst" => 'j'});

    let mut src_col = Vec::with_capacity(e as usize);
    let mut dst_col = Vec::with_capacity(e as usize);
    let mut rng = rand::thread_rng();
    for i in 0..e {
        // Choose two random, non-equal numbers within 1..n.
        let src = rng.gen_range(1, n + 1);
        let mut dst = rng.gen_range(1, n + 1);
        while src == dst {
            dst = rng.gen_range(1, n + 1);
        }
        src_col.push(src);
        dst_col.push(dst);
    }

    t.insert(&mut map!{"src" => src_col, "dst" => dst_col});
    t.set_crk_col("src");
    let t_count = t.count;
    t.rearrange(deal(t_count).iter());
    t
}

/* BENCHMARK-BFS:
    Run and print in csv-format benchmarks for BFS runs.
*/

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
    print!(",{}", start.to(end));
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
            let neighbours = (*(selection.get_i64_col("dst"))).v.clone();
            for dst in neighbours {
                discover(dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}

fn unoptimised_bfs(adjacency_list: &mut Table, start_node: i64) -> Vec<i64> {
    let src_col = adjacency_list.get_i64_col("src").v.clone();
    let dst_col = adjacency_list.get_i64_col("dst").v.clone();
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
    let mut src_col = adjacency_list.get_i64_col("src").v.clone();
    let mut dst_col = adjacency_list.get_i64_col("dst").v.clone();
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
    let src_col = adjacency_list.get_i64_col("src").v.clone();
    let dst_col = adjacency_list.get_i64_col("dst").v.clone();

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

/* PAGERANK
    Given an adjacency list and a vector of pageranks, where every pagerank is initialised to 1/|V|,
    perform an iterative computation of the pagerank until |PR(t) - PR(t-1)| < epsilon, for some
    defined and given error, epsilon. The damping factor is also given as a parameter.
    => prs[i] is the pagerank of node i.
*/

fn initialise_pageranks(n: i64) -> Vec<f64> {
    let initial_pr = (n as f64).recip();
    let mut pageranks: Vec<f64> = Vec::with_capacity(1 + n as usize);
    pageranks.push(0.0); // Nodes start at 1.
    for i in 0..n {
        pageranks.push(initial_pr);
    }
    pageranks
}

fn terminate(pageranks: &Vec<f64>, new_pageranks: &Vec<f64>, n: usize, epsilon: f64) -> bool {
    let mut sum_of_squared_differences = 0.0;
    for v in 1..n {
        let d = new_pageranks[v] - pageranks[v];
        let d_squared = d * d;
        sum_of_squared_differences += d_squared;
    }
    let e = sum_of_squared_differences.sqrt();
    e < epsilon
}

// Example from https://en.wikipedia.org/wiki/PageRank
fn pagerank_example_test<F>(mut pagerank: F) where F: FnMut(Table, &mut Vec<f64>, f64, f64, i64) -> Vec<f64> {
    let mut adjacency_list = Table::new();
    let n = 11;
    let src = vec![2, 3, 4, 4, 5, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 11];
    let dst = vec![3, 2, 1, 2, 2, 4, 6, 2, 5, 2, 5, 2, 5, 2, 5, 5,  5];
    adjacency_list.new_columns(map!{"src" => 'j', "dst" => 'j'});
    adjacency_list.insert(&mut map!{"src" => src, "dst" => dst});
    adjacency_list.set_crk_col("src");
    let mut pageranks = initialise_pageranks(11);
    pageranks = pagerank(adjacency_list, &mut pageranks, 0.85, 0.001, 50);
    let expected = vec![0.0, 0.02534, 0.29696, 0.26549, 0.03022, 0.06253, 0.03022, 0.01250, 0.01250, 0.01250, 0.01250, 0.01250];
    for i in 0..11 {
        if (pageranks[i] - expected[i]).abs() > 0.001 {
            println!("Failed!");
            print!("expected: ");pretty_println_f64vec(&expected);
            print!("actual:   ");pretty_println_f64vec(&pageranks);
            panic!()
        }
    }
    println!("Passed!");
}

fn inherit(inherited_rank: &mut f64, prw: f64, lw: i64) {
    let contribution = prw / (lw as f64);
    (*inherited_rank) += contribution;
}

fn unoptimised_pagerank(adjacency_list: Table, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let src_col = adjacency_list.get_i64_col("src").v.clone();
    let dst_col = adjacency_list.get_i64_col("dst").v.clone();
    let e = adjacency_list.count;
    let n = prs.len();
    let m = (1.0 - d) / (n as f64);
    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n + 1) {
        l.push(-1);
    }
    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();
    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;
            for i in 0..e {
                if dst_col[i] == v as i64 {
                    let w = src_col[i] as usize;
                    let lw = if l[w] == -1 { l[w] = src_col.iter().fold(0, |acc, x| acc + ((x == &(w as i64)) as i64)); l[w] } else { l[w] };
                    inherit(&mut inherited_rank, pageranks[w], lw);
                }
            }
            new_pageranks[v] = m + d * inherited_rank;
        }
        if terminate(&pageranks, &new_pageranks, n, epsilon) {
            break;
        }
        pageranks = new_pageranks.clone();
        iterations += 1;
        if iterations > max_iterations {
            break;
        }
    }
    new_pageranks
}

fn preclustered_pagerank(adjacency_list: Table, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let e = adjacency_list.count;
    let mut src_col = adjacency_list.get_i64_col("src").v.clone();
    let mut dst_col = adjacency_list.get_i64_col("dst").v.clone();

    // Cluster by dst column.
    let mut row_store = Vec::with_capacity(e);
    for i in 0..e {
        row_store.push((src_col[i], dst_col[i]));
    }
    row_store.sort_by_key(|&k| k.1);
    for i in 0..e {
        src_col[i] = row_store[i].0;
        dst_col[i] = row_store[i].1;
    }

    let n = prs.len();
    let m = (1.0 - d) / (n as f64);
    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n + 1) {
        l.push(-1);
    }
    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;

            let i;
            match dst_col.binary_search(&(v as i64)) {
                Ok(x)  => i = x,
                Err(_) => {
                    new_pageranks[v] = m;
                    continue;
                },
            }

            let mut inc_idx = i.clone();
            let mut dec_idx = i.clone();

            // Hits the correct nodes
            loop {
                let w = src_col[inc_idx] as usize;
                let lw = if l[w] == -1 { l[w] = src_col.iter().fold(0, |acc, x| acc + ((x == &(w as i64)) as i64)); l[w] } else { l[w] };
                inc_idx += 1;
                inherit(&mut inherited_rank, pageranks[w], lw);
                if inc_idx >= src_col.len() {
                    break;
                } else if dst_col[inc_idx] != (v as i64) {
                    break;
                }
            }
            while dec_idx > 0 {
                dec_idx -= 1;
                if dst_col[dec_idx] != (v as i64) {
                    break;
                }
                let w = src_col[dec_idx] as usize;
                let lw = if l[w] == -1 { l[w] = src_col.iter().fold(0, |acc, x| acc + ((x == &(w as i64)) as i64)); l[w] } else { l[w] };
                inherit(&mut inherited_rank, pageranks[w], lw);
            }
            new_pageranks[v] = m + d * inherited_rank;
        }
        if terminate(&pageranks, &new_pageranks, n, epsilon) {
            break;
        }
        pageranks = new_pageranks.clone();
        iterations += 1;
        if iterations > max_iterations {
            break;
        }
    }
    new_pageranks
}
