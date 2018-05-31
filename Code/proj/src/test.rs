use bit_vec::BitVec;

use std::collections::HashMap;

use overswap_rle_compression::*;
use cracker_index::AVLCrackerIndex;

#[test]
fn single_column_table_initialised_empty() {
    let table = OverswapRLETable::new();
    assert_eq!(table.count, 0);
}

#[test]
fn cracker_column_initialised_empty() {
    let table = OverswapRLETable::new();
    assert_eq!(table.crk_col.crk.len(), 0);
}

#[test]
fn can_create_table_with_three_columns() {
    let mut table = OverswapRLETable::new();
    table.new_columns(vec!["a", "b", "c"]);
    let mut keys = Vec::new();
    for key in table.columns.keys() {
        keys.push(key);
    }
    assert!(keys.contains(&&"a".to_string()));
    assert!(keys.contains(&&"b".to_string()));
    assert!(keys.contains(&&"c".to_string()));
}

#[test]
#[should_panic]
fn can_insert_into_multi_column_table() {
    let mut table = OverswapRLETable::new();
    table.new_columns(vec!["a", "b"]);
    table.insert(&mut map!{"a" => vec![1, 2, 3], "b" => vec![4, 5, 6]});
    assert_eq!(table.get_col("a").v, vec![1, 2, 3]);
    assert_eq!(table.get_col("b").v, vec![4, 5, 6]);
    table.get_col("c");
}

fn two_col_test_table() -> OverswapRLETable {
    let mut table = OverswapRLETable::new();
    table.new_columns(vec!["a", "b"]);
    table.insert(&mut map!{
            "a" => vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6],
            "b" => vec![1,  1,  0, 0, 0, 1,  0, 0, 1,  0, 1,  1,  0, 0]});
    table.set_crk_col("a");
    table
}

fn assert_base_column_equals(t: OverswapRLETable, column_name: &str, expected: Vec<i64>) {
    match t.get_col(column_name) {
        ref col => assert_eq!(col.v, expected),
    }
}

#[test]
fn can_index_into_multi_column_table() {
    let table = two_col_test_table();
    let selection = table.get_indices(vec![0, 1, 5, 8, 10, 11].iter());
    assert_base_column_equals(selection.clone(), "a", vec![13, 16, 12, 19, 14, 11]);
    assert_base_column_equals(selection.clone(), "b", vec![1, 1, 1, 1, 1, 1]);
}

#[test]
fn can_set_cracked_column() {
    let table = two_col_test_table();
    match table.get_col("a") {
        ref col => assert_eq!(table.crk_col.v, col.v),
    };
}

#[test]
fn can_rearrange_tuples() {
    let mut table = two_col_test_table();
    table.rearrange(vec![3, 5, 12, 6, 8, 13, 10, 9, 4, 11, 0, 1, 2, 7].iter());
    assert_base_column_equals(table.clone(), "a", vec![9, 12, 8, 7, 19, 6, 14, 3, 2, 11, 13, 16, 4, 1]);
    assert_base_column_equals(table.clone(), "b", vec![0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0]);
}

fn adjacency_list_table(src: Vec<i64>, dst: Vec<i64>) -> OverswapRLETable {
    let mut adjacency_list = OverswapRLETable::new();
    adjacency_list.new_columns(vec!["src", "dst"]);
    adjacency_list.insert(&mut map!{"src" => src, "dst" => dst});
    adjacency_list.set_crk_col("src");
    adjacency_list
}

#[test]
fn can_crack_in_three_for_single_value() {
    let mut adjacency_list
    = adjacency_list_table(vec![5, 2, 4, 1, 1, 4, 4, 3, 3, 1, 5, 2, 1, 2, 3, 3, 4, 5, 2, 5],
                           vec![3, 5, 5, 3, 4, 1, 2, 5, 2, 5, 2, 1, 2, 4, 1, 4, 3, 1, 3, 4]);
    let selection = adjacency_list.cracker_select_specific(3);
    assert_base_column_equals(selection.clone(), "src", vec![3, 3, 3, 3]);
    assert_base_column_equals(selection.clone(), "dst", vec![2, 1, 4, 5]);
    assert_eq!(selection.count, 4);
    assert_eq!(adjacency_list.crk_col.crk, vec![2, 2, 1, 1, 2, 1, 1, 2, 3, 3, 3, 3, 5, 4, 4, 4, 4, 5, 5, 5]);
}

#[test]
fn can_crack_in_three_for_single_value_out_of_lower_bound() {
    let mut adjacency_list = adjacency_list_table(vec![4, 4, 3, 3, 4, 4], vec![4, 2, 1, 4, 3, 5]);
    let selection = adjacency_list.cracker_select_specific(1);
    assert_base_column_equals(selection.clone(), "src", vec![]);
    assert_base_column_equals(selection.clone(), "dst", vec![]);
    assert_eq!(adjacency_list.crk_col.crk, vec![4, 4, 3, 3, 4, 4]);
}

#[test]
fn can_crack_in_three_for_single_value_out_of_upper_bound() {
    let mut adjacency_list = adjacency_list_table(vec![2, 2, 2, 4, 3, 2, 2], vec![3, 1, 2, 1, 5, 4, 4]);
    let selection = adjacency_list.cracker_select_specific(5);
    assert_base_column_equals(selection.clone(), "src", vec![]);
    assert_base_column_equals(selection.clone(), "dst", vec![]);
    assert_eq!(adjacency_list.crk_col.crk, vec![2, 2, 2, 4, 3, 2, 2]);
}

#[test]
fn can_exploit_cracker_index_for_selecting_single_value_medium_table() {
    let mut adjacency_list
    = adjacency_list_table(vec![3, 1, 5, 5, 1, 5, 2, 3, 1, 5, 5, 3],
                           vec![5, 3, 2, 1, 5, 1, 1, 4, 3, 1, 2, 5]);

    let selection_1 = adjacency_list.cracker_select_specific(5);
    assert_base_column_equals(selection_1.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_column_equals(selection_1.clone(), "dst", vec![1, 2, 1, 2, 1]);

    let selection_2 = adjacency_list.cracker_select_specific(2);
    assert_base_column_equals(selection_2.clone(), "src", vec![2]);
    assert_base_column_equals(selection_2.clone(), "dst", vec![1]);

    let selection_3 = adjacency_list.cracker_select_specific(1);
    assert_base_column_equals(selection_3.clone(), "src", vec![1, 1, 1]);
    assert_base_column_equals(selection_3.clone(), "dst", vec![3, 3, 5]);

    let selection_4 = adjacency_list.cracker_select_specific(3);
    assert_base_column_equals(selection_4.clone(), "src", vec![3, 3, 3]);
    assert_base_column_equals(selection_4.clone(), "dst", vec![4, 5, 5]);
    // After the BFS the cracker column should be fully clustered
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 1, 1, 2, 3, 3, 3, 5, 5, 5, 5, 5]);
}

#[test]
fn can_exploit_cracker_index_for_selecting_single_value_small_table() {
    let mut adjacency_list = adjacency_list_table(vec![4, 4, 4, 2, 4, 3],
                                                  vec![3, 3, 2, 1, 5, 4]);

    let selection_1 = adjacency_list.cracker_select_specific(3);
    assert_base_column_equals(selection_1.clone(), "src", vec![3]);
    assert_base_column_equals(selection_1.clone(), "dst", vec![4]);

    let selection_2 = adjacency_list.cracker_select_specific(4);
    assert_base_column_equals(selection_2.clone(), "src", vec![4, 4, 4, 4]);
    assert_base_column_equals(selection_2.clone(), "dst", vec![2, 3, 5, 3]);

    let selection_3 = adjacency_list.cracker_select_specific(2);
    assert_base_column_equals(selection_3.clone(), "src", vec![2]);
    assert_base_column_equals(selection_3.clone(), "dst", vec![1]);

    let selection_4 = adjacency_list.cracker_select_specific(5);
    assert_base_column_equals(selection_4.clone(), "src", vec![]);
    assert_base_column_equals(selection_4.clone(), "dst", vec![]);
    // After the BFS the cracker column should be fully clustered
    assert_eq!(adjacency_list.crk_col.crk, vec![2, 3, 4, 4, 4, 4]);
}

#[test]
fn repeat_queries_return_same_results() {
    let mut adjacency_list
    = adjacency_list_table(vec![3, 1, 5, 5, 1, 5, 2, 3, 1, 5, 5, 3],
                           vec![5, 3, 2, 1, 5, 1, 1, 4, 3, 1, 2, 5]);

    let selection_1 = adjacency_list.cracker_select_specific(5);
    assert_base_column_equals(selection_1.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_column_equals(selection_1.clone(), "dst", vec![1, 2, 1, 2, 1]);

    let selection_2 = adjacency_list.cracker_select_specific(5);
    assert_base_column_equals(selection_2.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_column_equals(selection_2.clone(), "dst", vec![1, 2, 1, 2, 1]);
}

#[test]
fn can_do_pagerank_iteration() {
    let d = 0.85;
    let n = 11;
    let m = 0.01363636; // = (1 - d) / n

    // Edge data
    let mut table = OverswapRLETable::new();
    table.new_columns(vec!["src", "dst"]);
    table.insert(&mut map!{"src" => vec![7, 4, 2, 3, 5, 9, 6, 5, 11, 4, 10, 8, 5, 6, 8, 7, 9],
                           "dst" => vec![2, 2, 3, 2, 4, 5, 5, 6, 5,  1, 5,  5, 2, 2, 2, 5, 2]});
    table.set_crk_col("dst");

    // Vertex data
    let prs = vec![0.0, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909, 0.09090909];
    let mut l = vec![-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1];

    let mut pageranks     = prs.clone();
    let mut new_pageranks = prs.clone();

    let mut in_degree = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for v in 1..n {
        let mut inherited_rank = 0.0;

        for w in table.cracker_select_specific(v as i64).get_col("src").v.iter().map(|&x|x as usize) {
            let lw = if l[w] == -1 { l[w] = (&table).count_col_eq("src", w as i64); l[w] } else { l[w] };
            in_degree[v] += 1;
            inherited_rank += pageranks[w] / (lw as f64);
        }

        new_pageranks[v] = m + d * inherited_rank;
    }
    pageranks = new_pageranks.clone();
    assert_eq!(l, vec![-1, -1, 1, 1, 2, 3, 2, 2, 2, 2, 1, 1]);
    assert_eq!(in_degree, vec![0, 1, 7, 1, 1, 6, 1, 0, 0, 0, 0, 0]);

    println!("Adjacency list:");
    table.print_cols();

    let one_dst_selection = table.cracker_select_specific(1);
    println!("Selection where dst=1:");
    one_dst_selection.print_cols();
    let one_in_neighbours: Vec<usize> = one_dst_selection.get_col("src").v.iter().map(|&x|x as usize).collect();
    assert_eq!(one_in_neighbours, vec![4]);

    let mut in_degree = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for v in 1..n {
        let mut inherited_rank = 0.0;

        for w in table.cracker_select_specific(v as i64).get_col("src").v.iter().map(|&x|x as usize) {
            let lw = if l[w] == -1 { l[w] = (&table).count_col_eq("src", w as i64); l[w] } else { l[w] };
            in_degree[v] += 1;
            inherited_rank += pageranks[w] / (lw as f64);
        }

        new_pageranks[v] = m + d * inherited_rank;
    }
    let _pageranks = new_pageranks.clone();
    assert_eq!(l, vec![-1, -1, 1, 1, 2, 3, 2, 2, 2, 2, 1, 1]);
    assert_eq!(in_degree, vec![0, 1, 7, 1, 1, 6, 1, 0, 0, 0, 0, 0]);
}

#[test]
fn can_crack_in_three_for_single_low_value() {
    let mut adjacency_list = adjacency_list_table(vec![1, 2, 4, 4, 3, 3, 4, 4], vec![2, 3, 4, 2, 1, 4, 3, 5]);
    let selection = adjacency_list.cracker_select_specific(2);
    assert_base_column_equals(selection.clone(), "src", vec![2]);
    assert_base_column_equals(selection.clone(), "dst", vec![3]);
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 2, 4, 4, 3, 3, 4, 4]);
}

#[test]
fn can_recognize_crk_idx_single_value_below_lower_bound() {
    let mut adjacency_list = adjacency_list_table(vec![1, 2, 4, 4, 3, 3, 4, 4], vec![2, 3, 4, 2, 1, 4, 3, 5]);
    let selection = adjacency_list.cracker_select_specific(3);
    assert_base_column_equals(selection.clone(), "src", vec![3, 3]);
    assert_base_column_equals(selection.clone(), "dst", vec![4, 1]);
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 2, 3, 3, 4, 4, 4, 4]);

    let selection = adjacency_list.cracker_select_specific(0);
    assert_base_column_equals(selection.clone(), "src", vec![]);
    assert_base_column_equals(selection.clone(), "dst", vec![]);
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 2, 3, 3, 4, 4, 4, 4]);
}

#[test]
fn can_recognize_crk_idx_single_value_above_upper_bound() {
    let mut adjacency_list = adjacency_list_table(vec![1, 2, 4, 4, 3, 3, 4, 4], vec![2, 3, 4, 2, 1, 4, 3, 5]);
    let selection = adjacency_list.cracker_select_specific(4);
    assert_base_column_equals(selection.clone(), "src", vec![4, 4, 4, 4]);
    assert_base_column_equals(selection.clone(), "dst", vec![4, 2, 3, 5]);
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 2, 3, 3, 4, 4, 4, 4]);

    let selection = adjacency_list.cracker_select_specific(10);
    assert_base_column_equals(selection.clone(), "src", vec![]);
    assert_base_column_equals(selection.clone(), "dst", vec![]);
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 2, 3, 3, 4, 4, 4, 4]);
}

#[test]
fn cracker_index_test() {
    let mut crk_idx = AVLCrackerIndex::new();
    crk_idx.insert(16, 28);
    crk_idx.insert(17, 29);
    crk_idx.insert(23, 37);
    crk_idx.insert(24, 40);
    crk_idx.subtract_where_greater_than(20, 5);
    assert_eq!(crk_idx.get(23), Some(32));
    assert_eq!(crk_idx.get(24), Some(35));
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

pub fn bfs_test(n: i64, src_nodes: Vec<i64>, dst_nodes: Vec<i64>, start_node: i64) -> bool {
    let mut adjacency_list = from_adjacency_vectors(src_nodes, dst_nodes, "src");

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
            let neighbours = (*(selection.get_col("dst"))).v.clone();
            for dst in neighbours {
                discover(dst, &mut visited, &mut frontier);
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