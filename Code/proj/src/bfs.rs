use time::PreciseTime;
use bit_vec::BitVec;

use rand::Rng; // HMM
use rand;

use datagen;
use decomposed_cracking;
use recognitive_compression;
use compactive_compression;
use intrafragment_compression;

/* BFS:
    Given an adjacency list of two i64 vectors, SRC_NODE and DST_NODE, this function visits every
    node in the graph from START_NODE.

    Returns the nodes visited in the order in which they were visited.
*/

// Prints to stdout valid csv lines containing the results of bfs benchmarks..
pub fn benchmark_sparse_bfs_csv(graph_sizes: Vec<i64>) {
    println!("nodes,edges,density,unoptimised,preclustered,preclusteredRLE");
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
    println!();
}

// Times a given bfs function against a given adjacency list using a given start node.
fn time_bfs<F>(mut bfs: F, src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) where F: FnMut(Vec<i64>, Vec<i64>, i64) -> Vec<i64> {
    let start = PreciseTime::now();
    let _visited = bfs(src_node, dst_node, start_node);
    let end = PreciseTime::now();
    print!(",{}", start.to(end));
}

pub fn test_bfs_methods() {
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
    println!("Intraco");
    bfs_example_test(intraco_bfs);
}

// Example modified from https://en.wikipedia.org/wiki/PageRank to make the graph strongly connected
pub fn bfs_example_test<F>(mut bfs: F) where F: FnMut(Vec<i64>, Vec<i64>, i64) -> Vec<i64> {
    let n = 30;
    let src = vec![23, 16, 29, 27, 14, 25, 8, 23, 30, 27, 27, 20, 6, 20, 9, 6, 28, 10, 22, 14, 29, 6, 21, 1, 19, 13, 1, 11, 29, 7, 3, 27, 22, 2, 14, 3, 25, 12, 11, 29, 26, 27, 17, 15, 14, 27, 1, 24, 18, 6, 24, 27, 9, 6, 14, 5, 4, 23];
    let dst = vec![30, 23, 14, 22, 29, 20, 1, 22, 23, 25, 2, 25, 5, 21, 14, 15, 6, 14, 23, 6, 13, 14, 20, 27, 3, 29, 12, 24, 24, 9, 19, 17, 27, 27, 27, 6, 27, 1, 18, 26, 29, 1, 27, 6, 9, 14, 8, 29, 11, 3, 11, 4, 7, 28, 10, 6, 27, 16];
    let start_node = 16;

    let visited = bfs(src, dst, start_node);

    let mut failed = false;

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

fn unoptimised_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
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

fn preclustered_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let e = src_node.len();
    let mut src_col = src_node.clone();
    let mut dst_col = dst_node.clone();

    let mut row_store = Vec::with_capacity(e);
    for i in 0..e {
        row_store.push((src_col[i], dst_col[i]));
    }
    row_store.sort_by_key(|&k| k.0);
    for i in 0..e {
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

fn preclustered_rle_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
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

// Decomposed cracking
fn decracked_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = decomposed_cracking::from_adjacency_vectors(src_node, dst_node, "src");

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

// Recognitive compression
fn reco_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src_node, dst_node, "src");

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

// Compactive compression
fn coco_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = compactive_compression::from_adjacency_vectors(src_node, dst_node, "src");

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

// Intra-fragment compression
fn intraco_bfs(src_node: Vec<i64>, dst_node: Vec<i64>, start_node: i64) -> Vec<i64> {
    let mut adjacency_list = intrafragment_compression::from_adjacency_vectors(src_node, dst_node, "src");

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
            if src == 27 { adjacency_list.dbg_switch = true };
            let selection = adjacency_list.cracker_select_specific(src);
            let neighbours = (*(selection.get_col("dst"))).v.clone();
            for dst in neighbours {
                discover(dst, &mut visited, &mut frontier);
            }
        }
    }
    bv_where(visited)
}