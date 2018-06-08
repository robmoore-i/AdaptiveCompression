use bit_vec::BitVec;

use std::collections::HashMap;

use overswap_rle_compression;

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

pub fn bfs_test(n: i64, src_nodes: Vec<i64>, dst_nodes: Vec<i64>, start_node: i64) -> bool {
    let mut adjacency_list = overswap_rle_compression::from_adjacency_vectors(src_nodes, dst_nodes, "src");

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
    let visited = bv_where(visited);

    let mut failed = false;

    if visited.len() != n as usize {
        println!("Incorrect visitations: {:?}", visited);
        failed = true;
    }

    for i in 1..(n + 1) {
        if !visited.contains(&i) {
            println!("Result does not contain {}", i);
            failed = true;
        }
    }

    if failed {
        println!("Failed!");
        false
    } else {
        println!("Passed!");
        true
    }
}

#[test]
fn l_side_i_longer() {
    let src = vec![4, 16, 22, 8, 26, 13, 22, 4, 18, 12, 13, 22, 10, 14, 22, 8, 19, 29, 8, 8, 17, 18, 22, 5, 2, 28, 8, 12, 24, 13, 15, 21, 30, 6, 18, 25, 7, 9, 19, 19, 4, 3, 11, 17, 28, 10, 8, 28, 22, 11, 29, 1, 20, 30, 8, 23, 4, 27];
    let dst = vec![20, 18, 19, 6, 12, 22, 2, 11, 16, 26, 27, 25, 9, 4, 13, 30, 18, 30, 28, 13, 24, 19, 7, 19, 22, 8, 17, 29, 17, 8, 18, 4, 29, 8, 15, 22, 22, 10, 5, 22, 14, 8, 28, 8, 11, 22, 23, 1, 10, 4, 12, 28, 4, 8, 3, 8, 21, 13];
    let start_node = 7;
    assert!(bfs_test(30, src, dst, start_node));
}

#[test]
fn h_side_i_longer_overlap() {
    let src = vec![19, 19, 13, 28, 19, 21, 29, 18, 22, 28, 29, 12, 24, 1, 20, 19, 23, 5, 19, 3, 28, 30, 23, 16, 27, 18, 15, 17, 15, 18, 12, 18, 2, 11, 19, 12, 12, 18, 6, 13, 13, 15, 29, 28, 4, 13, 10, 7, 3, 22, 25, 28, 9, 22, 23, 8, 26, 14];
    let dst = vec![24, 15, 12, 27, 23, 18, 4, 12, 28, 20, 18, 30, 19, 13, 28, 5, 19, 19, 16, 13, 22, 12, 14, 19, 28, 6, 2, 13, 25, 21, 18, 7, 15, 29, 3, 13, 28, 29, 18, 3, 1, 19, 11, 12, 29, 17, 22, 18, 19, 9, 15, 26, 22, 10, 8, 23, 28, 23];
    let start_node = 9;
    assert!(bfs_test(30, src, dst, start_node));
}

#[test]
fn h_side_h_longer() {
    let src =  vec![7, 5, 23, 27, 13, 26, 11, 25, 17, 17, 12, 28, 14, 2, 22, 29, 6, 2, 8, 5, 2, 18, 14, 3, 4, 24, 17, 13, 2, 1, 14, 6, 9, 2, 18, 17, 14, 14, 5, 5, 5, 15, 14, 12, 17, 19, 30, 8, 2, 20, 21, 1, 4, 16, 17, 10, 14, 2];
    let dst = vec![17, 21, 8, 17, 11, 18, 13, 2, 14, 27, 14, 12, 4, 25, 14, 2, 17, 5, 17, 14, 24, 26, 17, 5, 14, 2, 6, 2, 1, 30, 10, 20, 5, 13, 14, 16, 5, 22, 9, 2, 3, 2, 18, 28, 8, 4, 1, 23, 29, 6, 5, 2, 19, 17, 7, 14, 12, 15];
    let start_node = 25;
    assert!(bfs_test(30, src, dst, start_node));
}

#[test]
fn h_side_i_longer() {
    let src = vec![33, 31, 12, 31, 14, 43, 9, 40, 25, 42, 6, 14, 39, 19, 11, 31, 10, 40, 14, 28, 3, 24, 45, 48, 8, 4, 18, 38, 17, 31, 19, 31, 47, 36, 48, 40, 5, 34, 16, 46, 8, 40, 2, 37, 35, 31, 24, 2, 21, 22, 35, 18, 31, 27, 39, 30, 26, 20, 50, 31, 1, 31, 2, 31, 21, 27, 13, 32, 40, 24, 34, 31, 21, 31, 31, 7, 23, 31, 40, 24, 12, 29, 49, 14, 31, 21, 24, 15, 4, 31, 1, 31, 21, 41, 2, 44, 32, 6];
    let dst = vec![40, 25, 31, 1, 11, 39, 24, 32, 31, 48, 24, 44, 24, 31, 14, 47, 19, 31, 21, 40, 4, 6, 8, 32, 12, 31, 50, 31, 2, 37, 10, 2, 31, 21, 42, 33, 2, 31, 24, 1, 45, 49, 17, 31, 15, 40, 39, 5, 41, 34, 21, 31, 14, 7, 43, 21, 31, 31, 18, 20, 31, 26, 31, 18, 36, 31, 6, 40, 29, 16, 22, 27, 35, 23, 19, 27, 31, 38, 28, 2, 8, 40, 40, 31, 12, 14, 9, 35, 3, 34, 46, 4, 30, 21, 24, 14, 48, 13];
    let start_node = 33;
    assert!(bfs_test(50, src, dst, start_node));
}

#[test]
fn h_side_i_longer_overlap_2() {
    let src = vec![22, 14, 5, 7, 11, 6, 9, 17, 28, 28, 16, 21, 5, 3, 5, 27, 10, 30, 20, 4, 28, 5, 8, 3, 3, 11, 13, 5, 6, 9, 25, 8, 2, 3, 18, 24, 29, 17, 15, 25, 28, 3, 3, 3, 12, 3, 11, 11, 11, 19, 5, 26, 1, 5, 6, 28, 23, 6];
    let dst = vec![5, 5, 22, 3, 16, 21, 28, 29, 9, 27, 11, 6, 28, 13, 14, 28, 3, 11, 8, 3, 11, 2, 3, 24, 12, 30, 3, 26, 5, 18, 5, 20, 5, 11, 9, 3, 17, 25, 6, 17, 23, 7, 8, 10, 3, 4, 28, 3, 19, 11, 25, 5, 6, 6, 15, 5, 28, 1];
    let start_node = 12;
    assert!(bfs_test(30, src, dst, start_node));
}

#[test]
fn l_side_i_longer_tessellating() {
    let src = vec![7, 14, 20, 26, 12, 21, 18, 6, 22, 2, 29, 29, 8, 2, 13, 15, 4, 27, 25, 28, 9, 30, 17, 9, 2, 24, 2, 29, 10, 20, 16, 2, 29, 2, 2, 2, 5, 2, 2, 22, 3, 16, 20, 2, 20, 19, 3, 2, 22, 1, 23, 2, 7, 20, 11, 13, 22, 3];
    let dst = vec![2, 13, 10, 16, 2, 20, 20, 7, 19, 27, 25, 1, 2, 3, 22, 2, 2, 2, 29, 2, 5, 2, 29, 20, 22, 22, 7, 3, 20, 21, 2, 30, 17, 16, 11, 12, 9, 23, 4, 13, 20, 26, 9, 8, 3, 22, 2, 15, 24, 29, 2, 28, 6, 18, 2, 14, 2, 29];
    let start_node = 1;
    assert!(bfs_test(30, src, dst, start_node));
}