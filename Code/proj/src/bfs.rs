use time::PreciseTime;
use bit_vec::BitVec;

use rand::Rng; // HMM
use rand;

use datagen;
use decomposed_cracking;
use recognitive_compression;
use compactive_compression;
use underswap_rle_compression;
use overswap_rle_compression;
use time::Duration;
use quicksort::*;

/* BFS:
    Given an adjacency list of two i64 vectors, SRC_NODE and DST_NODE, this function visits every
    node in the graph from START_NODE.

    Returns the nodes visited in the order in which they were visited.
*/

pub fn run() {
    let n = 1000;
    let (src, dst) = datagen::randomly_connected_tree(n);
        let start_node = rand::thread_rng().gen_range(1, n);
    let _visited = underswap_rle_bfs(src, dst, start_node);
}

// Benchmark for single BFS runs

// Prints to stdout valid csv lines containing the results of bfs benchmarks..
pub fn benchmark_sparse_bfs_csv(graph_sizes: Vec<i64>) {
    println!("nodes,edges,density,unoptimised,preclustered,preclusteredRLE,decomposed,recognitive,compactive,underswapRLE,overswapRLE");
    for n in graph_sizes {
        benchmark_sparse_bfs(n);
    }
}

// Given a number of nodes N, produces a sparse connected graph of that many nodes and gets runtime
// performance for each of adaptive, unoptimised and preclustering methods. It prints to stdout a
// line of a csv file.
fn benchmark_sparse_bfs(n: i64) {
    let (src, dst) = datagen::randomly_connected_tree(n);
    let start_node = rand::thread_rng().gen_range(1, n);
    let e = src.len();
    print!("{},{},{}", n, e, datagen::graph_density(n, e));
    time_bfs(unoptimised_bfs,      src.clone(), dst.clone(), start_node);
    time_bfs(preclustered_bfs,     src.clone(), dst.clone(), start_node);
    time_bfs(preclustered_rle_bfs, src.clone(), dst.clone(), start_node);
    time_bfs(decracked_bfs,        src.clone(), dst.clone(), start_node);
    time_bfs(reco_bfs,             src.clone(), dst.clone(), start_node);
    time_bfs(coco_bfs,             src.clone(), dst.clone(), start_node);
    time_bfs(underswap_rle_bfs,    src.clone(), dst.clone(), start_node);
    time_bfs(overswap_rle_bfs,     src.clone(), dst.clone(), start_node);
    println!();
}

// Times a given bfs function against a given adjacency list using a given start node.
fn time_bfs<F>(mut bfs: F, src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) where F: FnMut(Vec<i64>, Vec<i64>, i64) -> Vec<i64> {
    let start = PreciseTime::now();
    let _visited = bfs(src_node, dst_node, start_node);
    let end = PreciseTime::now();
    print!(",{}", start.to(end));
}

// Benchmark for a number BFS runs

pub fn benchmark_sparse_bfs_csv_n_runs(runs: usize, graph_sizes: Vec<i64>) {
    println!("nodes,edges,density,unoptimised,preclustered,preclusteredRLE,decomposed,recognitive,compactive,underswapRLE,overswapRLE");
    for n in graph_sizes {
        benchmark_sparse_bfs_i_runs(runs, n);
    }
}

// For a random n node graph, runs bfs i times.
fn benchmark_sparse_bfs_i_runs(i: usize, n: i64) {
    let (src, dst) = datagen::randomly_connected_tree(n);
    let mut start_nodes: Vec<i64> = Vec::new();
    for _ in 0..i {
        start_nodes.push(rand::thread_rng().gen_range(1, n));
    }
    let e = src.len();
    print!("{},{},{}", n, e, datagen::graph_density(n, e));
    time_bfs_n_runs(unoptimised_bfs_n,      i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(preclustered_bfs_n,     i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(preclustered_rle_bfs_n, i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(decracked_bfs_n,        i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(reco_bfs_n,             i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(coco_bfs_n,             i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(underswap_rle_bfs_n,    i, src.clone(), dst.clone(), start_nodes.clone());
    time_bfs_n_runs(overswap_rle_bfs_n,     i, src.clone(), dst.clone(), start_nodes.clone());
    println!();
}

fn time_bfs_n_runs<F>(mut bfs_n: F, n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) where F: FnMut(usize, Vec<i64>, Vec<i64>, Vec<i64>) {
    let start = PreciseTime::now();
    let _visited = bfs_n(n, src_node, dst_node, start_nodes);
    let end = PreciseTime::now();
    print!(",{}", start.to(end));
}


pub fn example_test_bfs_methods() {
    println!("Unoptimised");
    bfs_example_test(unoptimised_bfs);
    println!("Preclustered");
    bfs_example_test(preclustered_bfs);
    println!("Preclustered RLE");
    bfs_example_test(preclustered_rle_bfs);
    println!("Decracked");
    bfs_example_test(decracked_bfs);
    println!("Reco");
    bfs_example_test(reco_bfs);
    println!("Coco");
    bfs_example_test(coco_bfs);
    println!("Underswap RLE");
    bfs_example_test(underswap_rle_bfs);
    println!("Overswap RLE");
    bfs_example_test(overswap_rle_bfs);
}

pub fn bfs_example_test<F>(mut bfs: F) where F: FnMut(Vec<i64>, Vec<i64>, i64) -> Vec<i64> {
    let n = 30 as i64;
    let src = vec![4, 16, 22, 8, 26, 13, 22, 4, 18, 12, 13, 22, 10, 14, 22, 8, 19, 29, 8, 8, 17, 18, 22, 5, 2, 28, 8, 12, 24, 13, 15, 21, 30, 6, 18, 25, 7, 9, 19, 19, 4, 3, 11, 17, 28, 10, 8, 28, 22, 11, 29, 1, 20, 30, 8, 23, 4, 27];
    let dst = vec![20, 18, 19, 6, 12, 22, 2, 11, 16, 26, 27, 25, 9, 4, 13, 30, 18, 30, 28, 13, 24, 19, 7, 19, 22, 8, 17, 29, 17, 8, 18, 4, 29, 8, 15, 22, 22, 10, 5, 22, 14, 8, 28, 8, 11, 22, 23, 1, 10, 4, 12, 28, 4, 8, 3, 8, 21, 13];
    let start_node = 7;

    let visited = bfs(src, dst, start_node);

    let mut failed = false;

    if visited.len() != n as usize {
        println!("Incorrect visitations: {:?}", visited);
        failed = true;
    }

    for i in 1..(n + 1) {
        if !visited.contains(&i) {
            println!("FAILED: Result {:?} does not contain {}", visited, i);
            failed = true;
        }
    }

    if failed {
        println!("Failed!");
    } else {
        println!("Passed!")
    }
}

pub fn bait() {
    let n = 30 as i64;

    let (src, dst) = datagen::randomly_connected_tree(n);
    let start_node = rand::thread_rng().gen_range(1, n);
//    println!("let src = vec!{:?};", src);
//    println!("let dst = vec!{:?};", dst);
//    println!("let start_node = {};", start_node);

    let visited = coco_bfs(src, dst, start_node);
    let mut failed = false;
    if visited.len() != n as usize {
        println!("Incorrect visitations: {:?}", visited);
        failed = true;
    }
    for i in 1..(n + 1) {
        if !visited.contains(&i) {
            println!("FAILED: Result does not contain {}", i);
            failed = true;
        }
    }
    if failed {
        println!("Failed!");
    }
}

pub fn random_test_bfs_methods() {
    let n = 100 as i64;
    let (src, dst) = datagen::randomly_connected_tree(n);
    let start_node = rand::thread_rng().gen_range(1, n);
    println!("src: {:?}", src);
    println!("dst: {:?}", dst);

    println!("Unoptimised");
    bfs_random_test(unoptimised_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Preclustered");
    bfs_random_test(preclustered_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Preclustered RLE");
    bfs_random_test(preclustered_rle_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Decracked");
    bfs_random_test(decracked_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Reco");
    bfs_random_test(reco_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Coco");
    bfs_random_test(coco_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Underswap RLE");
    bfs_random_test(underswap_rle_bfs, n, src.clone(), dst.clone(), start_node);
    println!("Overswap RLE");
    bfs_random_test(overswap_rle_bfs, n, src.clone(), dst.clone(), start_node);
}

pub fn bfs_random_test<F>(mut bfs: F, n: i64, src: Vec<i64>, dst: Vec<i64>, start_node: i64) where F: FnMut(Vec<i64>, Vec<i64>, i64) -> Vec<i64> {
    let visited = bfs(src, dst, start_node);
    let mut failed = false;
    if visited.len() != n as usize {
        println!("Incorrect visitations: {:?}", visited);
        failed = true;
    }
    for i in 1..(n + 1) {
        if !visited.contains(&i) {
            println!("FAILED: Result {:?} does not contain {}", visited, i);
            failed = true;
        }
    }
    if failed {
        println!("Failed!");
    } else {
        println!("Passed!")
    }
}

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

// Naive/Unoptimised
fn unoptimised_bfs_refs(src_node: &Vec<i64>, dst_node: &Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            for i in 0..src_node.len() {
                if src_node[i] == src {
                    discover(dst_node[i], &mut visited, &mut frontier);
                }
            }
        }
    }
    bv_where(visited)
}
fn unoptimised_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    unoptimised_bfs_refs(&src_node, &dst_node, start_node)
}
fn unoptimised_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    for i in 0..n {
        unoptimised_bfs_refs(&src_node, &dst_node, start_nodes[i]);
    }
}

// Preclustered
pub fn precluster(src_node: &Vec<i64>, dst_node: &Vec<i64>) -> (Vec<i64>, Vec<i64>) {
    let e = src_node.len();
    let mut src_col = src_node.clone();
    let mut dst_col = dst_node.clone();

    let mut row_store = Vec::with_capacity(e);
    for i in 0..e {
        row_store.push((src_col[i], dst_col[i]));
    }
    quicksort_by(&mut row_store, |&a, b| (a.0).cmp(&b.0));
    // row_store.sort_by_key(|&k| k.0);
    for i in 0..e {
        src_col[i] = row_store[i].0;
        dst_col[i] = row_store[i].1;
    }

    (src_col, dst_col)
}
fn preclustered_bfs_from_ready(src_col: &Vec<i64>, dst_col: &Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let binary_search_result = src_col.binary_search(&src);
            if binary_search_result.is_err() {
                continue;
            }
            let i = binary_search_result.unwrap();
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
fn preclustered_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let (src_col, dst_col) = precluster(&src_node, &dst_node);
    preclustered_bfs_from_ready(&src_col, &dst_col, start_node)
}
fn preclustered_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let (src_col, dst_col) = precluster(&src_node, &dst_node);
    for i in 0..n {
        preclustered_bfs_from_ready(&src_col, &dst_col, start_nodes[i]);
    }
}

// Preclustered RLE
fn precluster_and_rle(src_node: Vec<i64>, dst_node: Vec<i64>) -> Vec<Vec<i64>> {
    let mut encoded_col: Vec<Vec<i64>> = Vec::new();
    let n = src_node.len();
    for i in 0..n {
        let src_as_usize = src_node[i] as usize;
        let dst = dst_node[i];

        while encoded_col.len() <= src_as_usize {
            encoded_col.push(Vec::new());
        }

        if encoded_col[src_as_usize].is_empty() {
            encoded_col[src_as_usize] = vec![dst];
        } else {
            encoded_col[src_as_usize].push(dst);
        }
    }
    encoded_col
}
fn preclustered_rle_bfs_from_ready(encoded_col: &Vec<Vec<i64>>, start_node: i64) -> Vec<i64> {
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
fn preclustered_rle_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let encoded_col = precluster_and_rle(src_node, dst_node);
    preclustered_rle_bfs_from_ready(&encoded_col, start_node)
}
fn preclustered_rle_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let encoded_col = precluster_and_rle(src_node, dst_node);
    for i in 0..n {
        preclustered_rle_bfs_from_ready(&encoded_col, start_nodes[i]);
    }
}

// Decomposed cracking
fn decracked_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = decomposed_cracking::from_adjacency_vectors(src_node, dst_node, "src");
    decracked_bfs_adjl(&mut adjacency_list, start_node)
}
pub fn decracked_bfs_adjl(adjacency_list: &mut decomposed_cracking::DeCrackedTable, start_node: i64) -> Vec<i64> {
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
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}
fn decracked_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let mut adjacency_list = decomposed_cracking::from_adjacency_vectors(src_node, dst_node, "src");
    for i in 0..n {
        decracked_bfs_adjl(&mut adjacency_list, start_nodes[i]);
    }
}

// Recognitive compression
fn reco_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src_node, dst_node, "src");
    reco_bfs_adjl(&mut adjacency_list, start_node)
}
fn reco_bfs_adjl(adjacency_list: &mut recognitive_compression::ReCoTable, start_node: i64) -> Vec<i64> {
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
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}
fn reco_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src_node, dst_node, "src");
    for i in 0..n {
        reco_bfs_adjl(&mut adjacency_list, start_nodes[i]);
    }
}

// Compactive compression
fn coco_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = compactive_compression::from_adjacency_vectors(src_node, dst_node, "src");
    coco_bfs_adjl(&mut adjacency_list, start_node)
}
fn coco_bfs_adjl(adjacency_list: &mut compactive_compression::CoCoTable, start_node: i64) -> Vec<i64> {
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
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}
fn coco_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let mut adjacency_list = compactive_compression::from_adjacency_vectors(src_node, dst_node, "src");
    for i in 0..n {
        coco_bfs_adjl(&mut adjacency_list, start_nodes[i]);
    }
}


// Underswap-RLE compression
fn underswap_rle_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = underswap_rle_compression::from_adjacency_vectors(src_node, dst_node, "src");
    underswap_rle_bfs_adjl(&mut adjacency_list, start_node)
}
pub fn underswap_rle_bfs_adjl(adjacency_list: &mut underswap_rle_compression::UnderswapRLETable, start_node: i64) -> Vec<i64> {
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
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}
fn underswap_rle_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let mut adjacency_list = underswap_rle_compression::from_adjacency_vectors(src_node, dst_node, "src");
    for i in 0..n {
        underswap_rle_bfs_adjl(&mut adjacency_list, start_nodes[i]);
    }
}

// Overswap-RLE compression
fn overswap_rle_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = overswap_rle_compression::from_adjacency_vectors(src_node, dst_node, "src");
    overswap_rle_bfs_adjl(&mut adjacency_list, start_node)
}
pub fn overswap_rle_bfs_adjl(adjacency_list: &mut overswap_rle_compression::OverswapRLETable, start_node: i64) -> Vec<i64> {
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
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}
fn overswap_rle_bfs_n(n: usize, src_node: Vec<i64>, dst_node: Vec<i64>, start_nodes: Vec<i64>) {
    let mut adjacency_list = overswap_rle_compression::from_adjacency_vectors(src_node, dst_node, "src");
    for i in 0..n {
        overswap_rle_bfs_adjl(&mut adjacency_list, start_nodes[i]);
    }
}

// == BREAK-EVEN POINT ==
// Returns number of queries completed before (d) elapses.

pub fn decracked_bfs_adjl_until(mut adjacency_list: decomposed_cracking::DeCrackedTable, start_node: i64, d: Duration) -> usize {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    let start = PreciseTime::now();
    let mut queries = 0;
    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
            queries += 1;
            if start.to(PreciseTime::now()) > d {
                return queries;
            }
        }
    }
    queries
}

pub fn reco_bfs_adjl_until(mut adjacency_list: recognitive_compression::ReCoTable, start_node: i64, d: Duration) -> usize {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    let start = PreciseTime::now();
    let mut queries = 0;
    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
            queries += 1;
            if start.to(PreciseTime::now()) > d {
                return queries;
            }
        }
    }
    queries
}

pub fn coco_bfs_adjl_until(mut adjacency_list: compactive_compression::CoCoTable, start_node: i64, d: Duration) -> usize {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    let start = PreciseTime::now();
    let mut queries = 0;
    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
            queries += 1;
            if start.to(PreciseTime::now()) > d {
                return queries;
            }
        }
    }
    queries
}

pub fn underswap_bfs_adjl_until(mut adjacency_list: underswap_rle_compression::UnderswapRLETable, start_node: i64, d: Duration) -> usize {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    let start = PreciseTime::now();
    let mut queries = 0;
    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
            queries += 1;
            if start.to(PreciseTime::now()) > d {
                return queries;
            }
        }
    }
    queries
}

pub fn overswap_bfs_adjl_until(mut adjacency_list: overswap_rle_compression::OverswapRLETable, start_node: i64, d: Duration) -> usize {
    let mut frontier = vec![start_node];
    let mut visited = BitVec::from_elem(start_node as usize, false);

    let start = PreciseTime::now();
    let mut queries = 0;
    while !frontier.is_empty() {
        set_indices(&mut visited, indicise(frontier.clone()));

        let prev_frontier = frontier.clone();
        frontier.clear();
        for src in prev_frontier {
            let neighbours = adjacency_list.cracker_select_specific(src, "dst");
            for dst in &neighbours {
                discover(*dst, &mut visited, &mut frontier);
            }
            queries += 1;
            if start.to(PreciseTime::now()) > d {
                return queries;
            }
        }
    }
    queries
}