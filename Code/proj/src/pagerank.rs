use utils;

use recognitive_compression;
use compactive_compression;
use intrafragment_compression;

/* PAGERANK
    Given an adjacency list of two i64 vecs, SRC_NODE and DST_NODE and a vector of PAGERANKS, where
    every pagerank is initialised to 1/|V|, perform an iterative computation of the pagerank until
    |PR(t) - PR(t-1)| < EPSILON. The damping factor, D is also given as a parameter.
    Returns an f64 vector such that the ith element is the pagerank of node i.
*/

fn initialise_pageranks(n: usize) -> Vec<f64> {
    let initial_pr = (n as f64).recip();
    let mut pageranks: Vec<f64> = Vec::with_capacity(1 + n);
    pageranks.push(0.0); // Nodes start at 1, so put empty pagerank in position 0.
    for _ in 0..n {
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

pub fn test_pagerank_methods() {
    println!("Unoptimised");
    pagerank_example_test(unoptimised_pagerank);
    println!("Preclustered");
    pagerank_example_test(preclustered_pagerank);
    println!("Preclustered RLE");
    pagerank_example_test(preclustered_rle_pagerank);
    println!("Decracked");
    pagerank_example_test(decracked_pagerank);
    println!("Reco");
    pagerank_example_test(reco_pagerank);
    println!("Coco");
    pagerank_example_test(coco_pagerank);
    println!("Intraco");
    pagerank_example_test(intraco_pagerank);
}

// Example from https://en.wikipedia.org/wiki/PageRank
fn pagerank_example_test<F>(mut pagerank: F) where F: FnMut(Vec<i64>, Vec<i64>, &mut Vec<f64>, f64, f64, i64) -> Vec<f64> {
    let src = vec![2, 3, 4, 4, 5, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 11];
    let dst = vec![3, 2, 1, 2, 2, 4, 6, 2, 5, 2, 5, 2, 5, 2, 5, 5,  5];
    let n = 11; // Number of nodes.
    let mut pageranks = initialise_pageranks(n);
    let d = 0.85;
    let epsilon = 0.001;
    let max_iters = 50;
    pageranks = pagerank(src, dst, &mut pageranks, d, epsilon, max_iters);
    let expected = vec![0.0, 0.02534, 0.29696, 0.26549, 0.03022, 0.06253, 0.03022, 0.01250, 0.01250, 0.01250, 0.01250, 0.01250];
    let delta = 0.001;
    for i in 0..n {
        if (pageranks[i] - expected[i]).abs() > delta {
            println!("Failed!");
            print!("expected: ");utils::pretty_println_f64vec(&expected);
            print!("actual:   ");utils::pretty_println_f64vec(&pageranks);
            panic!()
        }
    }
    println!("Passed!");
}

fn inherit(inherited_rank: &mut f64, prw: f64, lw: i64) {
    let contribution = prw / (lw as f64);
    (*inherited_rank) += contribution;
}

fn unoptimised_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let e = src_node.len();
    let n = prs.len();
    let m = (1.0 - d) / (n as f64);

    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n + 1) { l.push(-1); }

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();
    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;
            for i in 0..e {
                if dst_node[i] == v as i64 {
                    let w = src_node[i] as usize;
                    let lw = if l[w] == -1 { l[w] = src_node.iter().fold(0, |acc, x| acc + ((x == &(w as i64)) as i64)); l[w] } else { l[w] };
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

fn preclustered_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let e = src_node.len();
    let mut src_col = src_node.clone();
    let mut dst_col = dst_node.clone();

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
    for _ in 0..(n + 1) { l.push(-1); }

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

fn preclustered_rle_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let e = src_node.len();
    let n = prs.len();
    let m = (1.0 - d) / (n as f64);

    // Cluster with RLE by the dst column.
    let mut encoded_col: Vec<Vec<i64>> = Vec::with_capacity(n + 1);
    for i in 0..e {
        let dst_as_usize = dst_node[i] as usize;
        let src = src_node[i];

        while encoded_col.len() <= dst_as_usize {
            encoded_col.push(Vec::new());
        }

        if encoded_col[dst_as_usize].is_empty() {
            encoded_col[dst_as_usize] = vec![src];
        } else {
            encoded_col[dst_as_usize].push(src);
        }
    }
    while encoded_col.len() < n {
        encoded_col.push(vec![]);
    }

    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n + 1) { l.push(-1); }

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;

            let in_neighbours = encoded_col[v].iter().map(|&x|x as usize);
            for w in in_neighbours {
                let lw = if l[w] == -1 { l[w] = src_node.iter().fold(0, |acc, x| acc + ((x == &(w as i64)) as i64)); l[w] } else { l[w] };
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

// Decomposed cracking
fn decracked_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src_node, dst_node, "dst");

    let n = prs.len();
    let m = (1.0 - d) / (n as f64);

    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n+1) { l.push(-1); }

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;

            for w in adjacency_list.cracker_select_specific(v as i64).get_i64_col("src").v.iter().map(|&x|x as usize) {
                let lw = if l[w] == -1 { l[w] = (&adjacency_list).count_col_eq("src", w as i64); l[w] } else { l[w] };
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

// Recognitive compression
fn reco_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src_node, dst_node, "dst");

    let n = prs.len();
    let m = (1.0 - d) / (n as f64);

    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n+1) { l.push(-1); }

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;

            for w in adjacency_list.cracker_select_specific(v as i64).get_i64_col("src").v.iter().map(|&x|x as usize) {
                let lw = if l[w] == -1 { l[w] = (&adjacency_list).count_col_eq("src", w as i64); l[w] } else { l[w] };
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

// Compactive compression
fn coco_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let mut adjacency_list = compactive_compression::from_adjacency_vectors(src_node, dst_node, "dst");

    let n = prs.len();
    let m = (1.0 - d) / (n as f64);

    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n+1) { l.push(-1); }

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut iterations = 0;
    loop {

        for v in 1..n {
            let mut inherited_rank = 0.0;

            for w in adjacency_list.cracker_select_specific(v as i64).get_i64_col("src").v.iter().map(|&x|x as usize) {
                let lw = if l[w] == -1 { l[w] = (&adjacency_list).count_col_eq("src", w as i64); l[w] } else { l[w] };
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

// Intra-fragment compression
fn intraco_pagerank(src_node: Vec<i64>, dst_node: Vec<i64>, prs: &mut Vec<f64>, d: f64, epsilon: f64, max_iterations: i64) -> Vec<f64> {
    let mut adjacency_list = intrafragment_compression::from_adjacency_vectors(src_node, dst_node, "dst");

    let n = prs.len();
    let m = (1.0 - d) / (n as f64);

    let mut l = Vec::with_capacity(1 + n);
    for _ in 0..(n+1) { l.push(-1); }

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut iterations = 0;
    loop {
        for v in 1..n {
            let mut inherited_rank = 0.0;

            for w in adjacency_list.cracker_select_specific(v as i64).get_col("src").v.iter().map(|&x|x as usize) {
                let lw = if l[w] == -1 { l[w] = (&adjacency_list).count_col_eq("src", w as i64); l[w] } else { l[w] };
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