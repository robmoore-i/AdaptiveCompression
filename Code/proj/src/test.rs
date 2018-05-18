use std::collections::HashMap;

use underswap_rle_compression::UnderswapRLETable;
use cracker_index::AVLCrackerIndex;

#[test]
fn single_column_table_initialised_empty() {
    let table = UnderswapRLETable::new();
    assert_eq!(table.count, 0);
}

#[test]
fn cracker_column_initialised_empty() {
    let table = UnderswapRLETable::new();
    assert_eq!(table.crk_col.crk.len(), 0);
}

#[test]
fn can_create_table_with_three_columns() {
    let mut table = UnderswapRLETable::new();
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
    let mut table = UnderswapRLETable::new();
    table.new_columns(vec!["a", "b"]);
    table.insert(&mut map!{"a" => vec![1, 2, 3], "b" => vec![4, 5, 6]});
    assert_eq!(table.get_col("a").v, vec![1, 2, 3]);
    assert_eq!(table.get_col("b").v, vec![4, 5, 6]);
    table.get_col("c");
}

fn two_col_test_table() -> UnderswapRLETable {
    let mut table = UnderswapRLETable::new();
    table.new_columns(vec!["a", "b"]);
    table.insert(&mut map!{
            "a" => vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6],
            "b" => vec![1,  1,  0, 0, 0, 1,  0, 0, 1,  0, 1,  1,  0, 0]});
    table.set_crk_col("a");
    table
}

fn assert_base_column_equals(t: UnderswapRLETable, column_name: &str, expected: Vec<i64>) {
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

fn adjacency_list_table(src: Vec<i64>, dst: Vec<i64>) -> UnderswapRLETable {
    let mut adjacency_list = UnderswapRLETable::new();
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
    assert_base_column_equals(selection_1.clone(), "dst", vec![2, 1, 1, 2, 1]);

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
    assert_base_column_equals(selection_1.clone(), "dst", vec![2, 1, 1, 2, 1]);

    let selection_2 = adjacency_list.cracker_select_specific(5);
    assert_base_column_equals(selection_2.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_column_equals(selection_2.clone(), "dst", vec![2, 1, 1, 2, 1]);
}

#[test]
fn can_do_pagerank_iteration() {
    let d = 0.85;
    let n = 11;
    let m = 0.01363636; // = (1 - d) / n

    // Edge data
    let mut table = UnderswapRLETable::new();
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