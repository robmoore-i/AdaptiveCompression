use db::*;
use std::collections::HashMap;

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

// I credit these two macros (matches, _tt_as_expr_hack) to this chap:
// http://rrichardson.github.io/reactor/src/mac/matches.rs.html#18-27
#[macro_export]
macro_rules! matches {
        ($expr:expr, $($pat:tt)+) => {
             _tt_as_expr_hack! {
                match $expr {
                    $($pat)+ => true,
                    _        => false
                }
            }
        }
    }
// Work around "error: unexpected token: `an interpolated tt`", whatever that
// means. (Probably rust-lang/rust#22819.)
#[doc(hidden)]
#[macro_export]
macro_rules! _tt_as_expr_hack {
        ($value:expr) => ($value)
    }

fn one_col_test_table() -> Table {
    let mut table = Table::new();
    table.new_columns(map!{"a" => 'j'});
    table.insert(&mut map!{"a" => vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]});
    table.set_crk_col("a");
    table
}

#[test]
fn single_column_table_initialised_empty() {
    let table = Table::new();
    assert_eq!(table.count, 0);
}

#[test]
fn cracker_column_initialised_empty() {
    let table = Table::new();
    assert_eq!(table.crk_col.crk.len(), 0);
}

#[test]
fn cracker_select_in_three_from_single_column_table() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(10, 14, false, false);
        assert_eq!(selection.crk_col.v, vec![13, 12, 11]);
    }
    assert_eq!(table.crk_col.crk, vec![6, 4, 9, 2, 7, 1, 8, 3, 13, 12, 11, 14, 19, 16]);
}

#[test]
fn cracker_select_in_three_can_utilise_previous_queries() {
    let mut table = one_col_test_table();
    {
        table.cracker_select_in_three(10, 14, false, false);
        assert!(table.crk_col.crk_idx.contains(11));
        assert!(table.crk_col.crk_idx.contains(14));
        let selection = table.cracker_select_in_three(5, 10, false, false);
        assert!(table.crk_col.crk_idx.contains(6));
        assert!(table.crk_col.crk_idx.contains(10));
        assert_eq!(selection.crk_col.v, vec![7, 9, 8, 6]);
    }
    assert_eq!(table.crk_col.crk, vec![4, 2, 1, 3, 7, 9, 8, 6, 13, 12, 11, 14, 19, 16]);
}

#[test]
fn cracker_select_in_three_from_single_column_table_inc_low() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(3, 7, true, false);
        assert_eq!(selection.crk_col.v, vec![4, 6, 3]);
    }
    assert_eq!(table.crk_col.crk, vec![1, 2, 4, 6, 3, 12, 7, 9, 19, 16, 14, 11, 8, 13]);
}

#[test]
fn cracker_select_in_three_from_single_column_table_inc_high() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(13, 19, false, true);
        assert_eq!(selection.crk_col.v, vec![19, 16, 14]);
    }
    assert_eq!(table.crk_col.crk, vec![13, 4, 9, 2, 12, 7, 1, 3, 11, 8, 6, 19, 16, 14]);
}

#[test]
fn cracker_select_in_three_from_single_column_table_inc_both() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(1, 6, true, true);
        assert_eq!(selection.crk_col.v, vec![6, 3, 4, 1, 2]);
    }
    assert_eq!(table.crk_col.crk, vec![6, 3, 4, 1, 2, 12, 7, 9, 19, 16, 14, 11, 8, 13]);
}

#[test]
fn cracker_select_in_two_from_single_column_table() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_two(7, true);
        assert_eq!(selection.crk_col.v, vec![6, 3, 4, 1, 2, 7]);
    }
    assert_eq!(table.crk_col.crk, vec![6, 3, 4, 1, 2, 7, 12, 9, 19, 16, 14, 11, 8, 13]);
}

#[test]
fn cracker_select_in_two_from_single_column_table_not_inclusive() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_two(10, false);
        assert_eq!(selection.crk_col.v, vec![6, 8, 4, 9, 2, 3, 7, 1]);
    }
    assert_eq!(table.crk_col.crk, vec![6, 8, 4, 9, 2, 3, 7, 1, 19, 12, 14, 11, 16, 13]);
}

#[test]
fn cracker_select_in_two_can_utilise_previous_queries() {
    let mut table = one_col_test_table();
    {
        table.cracker_select_in_three(10, 14, false, false);
        let selection = table.cracker_select_in_two(7, false);
        assert_eq!(selection.crk_col.v, vec![6, 4, 3, 2, 1]);
    }
    assert_eq!(table.crk_col.crk, vec![6, 4, 3, 2, 1, 7, 8, 9, 13, 12, 11, 14, 19, 16]);
}

#[test]
fn cracker_select_in_three_after_crack_in_two() {
    let mut table = one_col_test_table();
    {
        table.cracker_select_in_two(7, true);
        let selection = table.cracker_select_in_three(6, 11, true, false);
        assert_eq!(selection.crk_col.v, vec![6, 7, 8, 9]);
    }
    assert_eq!(table.crk_col.crk, vec![3, 4, 1, 2, 6, 7, 8, 9, 19, 16, 14, 11, 12, 13]);
}

#[test]
fn crack_in_two_above_upper_limit() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_two(25, true);
        assert_eq!(selection.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
    }
    assert_eq!(table.crk_col.crk, [13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
}

#[test]
fn crack_in_two_below_lower_limit() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_two(-5, true);
        assert_eq!(selection.crk_col.v, vec![]);
    }
    assert_eq!(table.crk_col.crk, [13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
}

#[test]
fn crack_in_three_between_value_within_column_and_above_upper_limit() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(14, 25, true, false);
        assert_eq!(selection.crk_col.v, vec![19, 16, 14]);
    }
    assert_eq!(table.crk_col.crk, [13, 4, 9, 2, 12, 7, 1, 3, 11, 8, 6, 19, 16, 14]);
}

#[test]
fn crack_in_three_between_value_within_column_and_below_lower_limit() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(-5, 4, true, false);
        assert_eq!(selection.crk_col.v, vec![3, 1, 2]);
    }
    assert_eq!(table.crk_col.crk, [3, 1, 2, 9, 4, 12, 7, 16, 19, 13, 14, 11, 8, 6]);
}

#[test]
fn crack_in_three_select_entire_column() {
    let mut table = one_col_test_table();
    {
        let selection = table.cracker_select_in_three(-50, 200, false, false);
        assert_eq!(selection.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
    }
    assert_eq!(table.crk_col.crk, [13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
}

#[test]
fn can_crack_in_three_over_three_queries() {
    let mut table = one_col_test_table();
    {
        table.cracker_select_in_three(10, 14, false, false);
        let s1 = table.cracker_select_in_three(3, 11, false, true);
        assert_eq!(s1.crk_col.v, vec![6, 7, 4, 8, 9, 11]);
    }
    {
        let s2 = table.cracker_select_in_three(7, 17, true, false);
        assert_eq!(s2.crk_col.v, vec![7, 8, 9, 11, 12, 13, 14, 16]);
    }
    assert_eq!(table.crk_col.crk, [2, 1, 3, 6, 4, 7, 8, 9, 11, 12, 13, 14, 16, 19]);
}

#[test]
fn can_crack_in_two_over_three_queries() {
    let mut table = one_col_test_table();
    {
        let s1 = table.cracker_select_in_two(10, true);
        assert_eq!(s1.crk_col.v, vec![6, 8, 4, 9, 2, 3, 7, 1]);
    }
    {
        let s2 = table.cracker_select_in_two(3, true);
        assert_eq!(s2.crk_col.v, vec![1, 3, 2]);
    }
    {
        let s3 = table.cracker_select_in_two(14, false);
        assert_eq!(s3.crk_col.v, vec![1, 3, 2, 9, 4, 8, 7, 6, 13, 12, 11]);
    }
    assert_eq!(table.crk_col.crk, [1, 3, 2, 9, 4, 8, 7, 6, 13, 12, 11, 14, 16, 19]);
}

#[test]
fn cracker_index_handles_inclusivity_at_upper_bound() {
    let mut table = one_col_test_table();
    {
        let s1 = table.cracker_select_in_two(19, true);
        assert_eq!(s1.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
    }
    {
        let s2 = table.cracker_select_in_three(10, 19, false, true);
        assert_eq!(s2.crk_col.v, vec![19, 12, 14, 11, 16, 13]);
    }
    assert_eq!(table.crk_col.crk, [4, 9, 2, 7, 1, 3, 8, 6, 19, 12, 14, 11, 16, 13]);
}

#[test]
fn cracker_index_handles_inclusivity_close_to_upper_bound() {
    let mut table = one_col_test_table();
    {
        let s1 = table.cracker_select_in_two(19, false);
        assert_eq!(s1.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 6, 3, 14, 11, 8]);
    }
    {
        let s2 = table.cracker_select_in_three(10, 19, false, true);
        assert_eq!(s2.crk_col.v, vec![12, 16, 14, 11, 13, 19]);
    }
    assert_eq!(table.crk_col.crk, [4, 9, 2, 7, 1, 6, 3, 8, 12, 16, 14, 11, 13, 19]);
}

#[test]
fn cracker_index_handles_inclusivity_at_lower_bound() {
    let mut table = one_col_test_table();
    {
        let s1 = table.cracker_select_in_two(1, true);
        assert_eq!(s1.crk_col.v, vec![1]);
    }
    {
        let s2 = table.cracker_select_in_three(1, 5, true, true);
        assert_eq!(s2.crk_col.v, vec![1, 3, 4, 2]);
    }
    assert_eq!(table.crk_col.crk, [1, 3, 4, 2, 9, 12, 7, 13, 19, 16, 14, 11, 8, 6]);
}

#[test]
fn cracker_index_handles_inclusivity_close_to_lower_bound() {
    let mut table = one_col_test_table();
    {
        let s1 = table.cracker_select_in_two(2, false);
        assert_eq!(s1.crk_col.v, vec![1]);
    }
    {
        let s2 = table.cracker_select_in_three(1, 5, true, true);
        assert_eq!(s2.crk_col.v, vec![1, 3, 4, 2]);
    }
    assert_eq!(table.crk_col.crk, [1, 3, 4, 2, 9, 12, 7, 13, 19, 16, 14, 11, 8, 6]);
}

#[test]
fn can_create_table_with_three_columns() {
    let mut table = Table::new();
    table.new_columns(map!{"a" => 'j', "b" => 'j', "c" => 'j'});
    let mut keys = Vec::new();
    for key in table.i64_columns.keys() {
        keys.push(key);
    }
    assert!(keys.contains(&&"a".to_string()));
    assert!(keys.contains(&&"b".to_string()));
    assert!(keys.contains(&&"c".to_string()));
}

#[test]
#[should_panic]
fn can_insert_into_multi_column_table() {
    let mut table = Table::new();
    table.new_columns(map!{"a" => 'j', "b" => 'j'});
    table.insert(&mut map!{"a" => vec![1, 2, 3], "b" => vec![4, 5, 6]});
    assert_eq!(table.get_i64_col("a").v, vec![1, 2, 3]);
    assert_eq!(table.get_i64_col("b").v, vec![4, 5, 6]);
    table.get_i64_col("c");
}

fn two_col_test_table() -> Table {
    let mut table = Table::new();
    table.new_columns(map!{"a" => 'j', "b" => 'j'});
    table.insert(&mut map!{
            "a" => vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6],
            "b" => vec![1,  1,  0, 0, 0, 1,  0, 0, 1,  0, 1,  1,  0, 0]});
    table.set_crk_col("a");
    table
}

fn assert_base_i64_column_equals(t: Table, column_name: &str, expected: Vec<i64>) {
    match t.get_i64_col(column_name) {
        ref col => assert_eq!(col.v, expected),
    }
}

fn assert_base_f64_column_equals(t: Table, column_name: &str, expected: Vec<f64>) {
    match t.get_f64_col(column_name) {
        ref col => assert_eq!(col.v, expected),
    }
}

#[test]
fn can_index_into_multi_column_table() {
    let table = two_col_test_table();
    let selection = table.get_indices(vec![0, 1, 5, 8, 10, 11].iter());
    assert_base_i64_column_equals(selection.clone(), "a", vec![13, 16, 12, 19, 14, 11]);
    assert_base_i64_column_equals(selection.clone(), "b", vec![1, 1, 1, 1, 1, 1]);
}

#[test]
fn can_set_cracked_column() {
    let table = two_col_test_table();
    match table.get_i64_col("a") {
        ref col => assert_eq!(table.crk_col.v, col.v),
    };
}

#[test]
fn crack_returns_indices_into_base_columns() {
    let mut table = two_col_test_table();
    let selection = table.cracker_select_in_three(10, 14, false, false);
    assert_base_i64_column_equals(selection.clone(), "a", vec![13, 12, 11]);
    assert_base_i64_column_equals(selection.clone(), "b", vec![1, 1, 1]);
}

#[test]
fn can_rearrange_tuples() {
    let mut table = two_col_test_table();
    table.rearrange(vec![3, 5, 12, 6, 8, 13, 10, 9, 4, 11, 0, 1, 2, 7].iter());
    assert_base_i64_column_equals(table.clone(), "a", vec![9, 12, 8, 7, 19, 6, 14, 3, 2, 11, 13, 16, 4, 1]);
    assert_base_i64_column_equals(table.clone(), "b", vec![0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0]);
}

fn adjacency_list_table(src: Vec<i64>, dst: Vec<i64>) -> Table {
    let mut adjacency_list = Table::new();
    adjacency_list.new_columns(map!{"src" => 'j', "dst" => 'j'});
    adjacency_list.insert(&mut map!{"src" => src, "dst" => dst});
    adjacency_list.set_crk_col("src");
    adjacency_list
}

#[test]
fn can_crack_in_three_for_single_value() {
    let mut adjacency_list
    = adjacency_list_table(vec![5, 2, 4, 1, 1, 4, 4, 3, 3, 1, 5, 2, 1, 2, 3, 3, 4, 5, 2, 5],
                           vec![3, 5, 5, 3, 4, 1, 2, 5, 2, 5, 2, 1, 2, 4, 1, 4, 3, 1, 3, 4]);
    let selection = adjacency_list.cracker_select_in_three(3, 3, true, true);
    assert_base_i64_column_equals(selection.clone(), "src", vec![3, 3, 3, 3]);
    assert_base_i64_column_equals(selection.clone(), "dst", vec![2, 1, 4, 5]);
    assert_eq!(selection.count, 4);
    assert_eq!(adjacency_list.crk_col.crk, vec![2, 2, 1, 1, 2, 1, 1, 2, 3, 3, 3, 3, 5, 4, 4, 4, 4, 5, 5, 5]);
}

#[test]
fn can_crack_in_three_for_single_value_out_of_lower_bound() {
    let mut adjacency_list = adjacency_list_table(vec![4, 4, 3, 3, 4, 4], vec![4, 2, 1, 4, 3, 5]);
    let selection = adjacency_list.cracker_select_in_three(1, 1, true, true);
    assert_base_i64_column_equals(selection.clone(), "src", vec![]);
    assert_base_i64_column_equals(selection.clone(), "dst", vec![]);
    assert_eq!(adjacency_list.crk_col.crk, vec![4, 4, 3, 3, 4, 4]);
}

#[test]
fn can_crack_in_three_for_single_value_out_of_upper_bound() {
    let mut adjacency_list = adjacency_list_table(vec![2, 2, 4, 3, 2, 2], vec![3, 2, 1, 5, 4, 4]);
    let selection = adjacency_list.cracker_select_in_three(5, 5, true, true);
    assert_base_i64_column_equals(selection.clone(), "src", vec![]);
    assert_base_i64_column_equals(selection.clone(), "dst", vec![]);
    assert_eq!(adjacency_list.crk_col.crk, vec![2, 2, 4, 3, 2, 2]);
}

#[test]
fn can_exploit_cracker_index_for_selecting_single_value_medium_table() {
    let mut adjacency_list
    = adjacency_list_table(vec![3, 1, 5, 5, 1, 5, 2, 3, 1, 5, 5, 3],
                           vec![5, 3, 2, 1, 5, 1, 1, 4, 3, 1, 2, 5]);

    let selection_1 = adjacency_list.cracker_select_in_three(5, 5, true, true);
    assert_base_i64_column_equals(selection_1.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_i64_column_equals(selection_1.clone(), "dst", vec![2, 1, 1, 2, 1]);

    let selection_2 = adjacency_list.cracker_select_in_three(2, 2, true, true);
    assert_base_i64_column_equals(selection_2.clone(), "src", vec![2]);
    assert_base_i64_column_equals(selection_2.clone(), "dst", vec![1]);

    let selection_3 = adjacency_list.cracker_select_in_three(1, 1, true, true);
    assert_base_i64_column_equals(selection_3.clone(), "src", vec![1, 1, 1]);
    assert_base_i64_column_equals(selection_3.clone(), "dst", vec![3, 3, 5]);

    let selection_4 = adjacency_list.cracker_select_in_three(3, 3, true, true);
    assert_base_i64_column_equals(selection_4.clone(), "src", vec![3, 3, 3]);
    assert_base_i64_column_equals(selection_4.clone(), "dst", vec![4, 5, 5]);
    // After the BFS the cracker column should be fully clustered
    assert_eq!(adjacency_list.crk_col.crk, vec![1, 1, 1, 2, 3, 3, 3, 5, 5, 5, 5, 5]);
}

#[test]
fn can_exploit_cracker_index_for_selecting_single_value_small_table() {
    let mut adjacency_list = adjacency_list_table(vec![4, 4, 4, 2, 4, 3],
                                                  vec![3, 3, 2, 1, 5, 4]);

    let selection_1 = adjacency_list.cracker_select_in_three(3, 3, true, true);
    assert_base_i64_column_equals(selection_1.clone(), "src", vec![3]);
    assert_base_i64_column_equals(selection_1.clone(), "dst", vec![4]);

    let selection_2 = adjacency_list.cracker_select_in_three(4, 4, true, true);
    assert_base_i64_column_equals(selection_2.clone(), "src", vec![4, 4, 4, 4]);
    assert_base_i64_column_equals(selection_2.clone(), "dst", vec![2, 3, 5, 3]);

    println!("src: {:?}", adjacency_list.crk_col.crk);
    println!("dst: {:?}", adjacency_list.get_i64_col("dst").v);

    let selection_3 = adjacency_list.cracker_select_in_three(2, 2, true, true);
    assert_base_i64_column_equals(selection_3.clone(), "src", vec![2]);
    assert_base_i64_column_equals(selection_3.clone(), "dst", vec![1]);

    let selection_4 = adjacency_list.cracker_select_in_three(5, 5, true, true);
    assert_base_i64_column_equals(selection_4.clone(), "src", vec![]);
    assert_base_i64_column_equals(selection_4.clone(), "dst", vec![]);
    // After the BFS the cracker column should be fully clustered
    assert_eq!(adjacency_list.crk_col.crk, vec![2, 3, 4, 4, 4, 4]);
}

#[test]
fn repeat_queries_return_same_results() {
    let mut adjacency_list
    = adjacency_list_table(vec![3, 1, 5, 5, 1, 5, 2, 3, 1, 5, 5, 3],
                           vec![5, 3, 2, 1, 5, 1, 1, 4, 3, 1, 2, 5]);

    let selection_1 = adjacency_list.cracker_select_specific(5);
    assert_base_i64_column_equals(selection_1.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_i64_column_equals(selection_1.clone(), "dst", vec![2, 1, 1, 2, 1]);

    let selection_2 = adjacency_list.cracker_select_specific(5);
    assert_base_i64_column_equals(selection_2.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_i64_column_equals(selection_2.clone(), "dst", vec![2, 1, 1, 2, 1]);
}

#[test]
fn can_create_new_float_columns() {
    let mut adjacency_list = Table::new();
    adjacency_list.new_columns(map!{"src" => 'j', "dst" => 'j', "pr" => 'f'});
    assert_eq!(adjacency_list.count, 0);
    assert_eq!(adjacency_list.i64_columns.len(), 2);
    assert_eq!(adjacency_list.f64_columns.len(), 1);
}

#[test]
fn can_get_float_column() {
    let mut adjacency_list = Table::new();
    adjacency_list.new_columns(map!{"src" => 'j', "dst" => 'j', "pr" => 'f'});
    let pr = adjacency_list.get_f64_col("pr");
    assert!(pr.v.is_empty());
}

#[test]
fn can_populate_float_column() {
    let mut adjacency_list = Table::new();
    adjacency_list.new_columns(map!{"src" => 'j', "dst" => 'j', "pr" => 'f'});
    let v = vec![0.1, 0.2, 0.3];
    adjacency_list.insert_multityped(&mut map!{"src" => vec![1, 2, 3], "dst" => vec![3, 2, 1]}, &mut map!{"pr" => v.clone()});
    let pr = adjacency_list.get_f64_col("pr");
    assert_eq!(pr.v, v);
}

fn pr_table(src: Vec<i64>, dst: Vec<i64>, pr: Vec<f64>) -> Table {
    let mut prt = Table::new();
    prt.new_columns(map!{"src" => 'j', "dst" => 'j', "pr" => 'f'});
    prt.insert_multityped(&mut map!{"src" => src, "dst" => dst}, &mut map!{"pr" => pr});
    prt.set_crk_col("src");
    prt
}

#[test]
fn float_can_crack_in_three_for_single_value() {
    let mut prt = pr_table(vec![5,    2,    4,     1,    1,    4,   4,    3,    3,    1,    5,    2,   1,    2,    3,    3,   4,    5,    2,    5],
                           vec![3,    5,    5,     3,    4,    1,   2,    5,    2,    5,    2,    1,   2,    4,    1,    4,   3,    1,    3,    4],
                           vec![0.89, 0.44, 0.078, 0.42, 0.62, 0.2, 0.81, 0.24, 0.55, 0.53, 0.94, 0.3, 0.44, 0.44, 0.73, 1.0, 0.74, 0.24, 0.57, 0.43]);
    let selection = prt.cracker_select_in_three(3, 3, true, true);
    assert_base_i64_column_equals(selection.clone(), "src", vec![3, 3, 3, 3]);
    assert_base_i64_column_equals(selection.clone(), "dst", vec![2, 1, 4, 5]);
    assert_base_f64_column_equals(selection.clone(), "pr",  vec![0.55, 0.73, 1.0, 0.24]);
    assert_eq!(selection.count, 4);
    assert_eq!(prt.crk_col.crk, vec![2, 2, 1, 1, 2, 1, 1, 2, 3, 3, 3, 3, 5, 4, 4, 4, 4, 5, 5, 5]);
}

#[test]
fn float_can_crack_in_three_for_single_value_out_of_lower_bound() {
    let mut prt = pr_table(vec![4,    4,     3,    3,     4,    4],
                           vec![4,    2,     1,    4,     3,    5],
                           vec![0.77, 0.016, 0.36, 0.025, 0.69, 0.64]);
    let selection = prt.cracker_select_in_three(1, 1, true, true);
    assert_base_i64_column_equals(selection.clone(), "src", vec![]);
    assert_base_i64_column_equals(selection.clone(), "dst", vec![]);
    assert_base_f64_column_equals(selection.clone(), "pr",  vec![]);
    assert_eq!(prt.crk_col.crk, vec![4, 4, 3, 3, 4, 4]);
}

#[test]
fn float_can_crack_in_three_for_single_value_out_of_upper_bound() {
    let mut prt = pr_table(vec![2,    2,     4,    3,     2,    2],
                           vec![3,    2,     1,    5,     4,    4],
                           vec![0.77, 0.016, 0.36, 0.025, 0.69, 0.64]);
    let selection = prt.cracker_select_in_three(5, 5, true, true);
    assert_base_i64_column_equals(selection.clone(), "src", vec![]);
    assert_base_i64_column_equals(selection.clone(), "dst", vec![]);
    assert_base_f64_column_equals(selection.clone(), "pr",  vec![]);
    assert_eq!(prt.crk_col.crk, vec![2, 2, 4, 3, 2, 2]);
}

#[test]
fn float_can_exploit_cracker_index_for_selecting_single_value_medium_table() {
    let mut prt = pr_table(vec![3,     1,   5,    5,   1,    5,     2,    3,    1,    5,    5,    3],
                           vec![5,     3,   2,    1,   5,    1,     1,    4,    3,    1,    2,    5],
                           vec![0.038, 0.9, 0.79, 0.2, 0.78, 0.069, 0.41, 0.23, 0.71, 0.14, 0.27, 0.64]);

    let selection_1 = prt.cracker_select_in_three(5, 5, true, true);
    assert_base_i64_column_equals(selection_1.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_i64_column_equals(selection_1.clone(), "dst", vec![2, 1, 1, 2, 1]);
    assert_base_f64_column_equals(selection_1.clone(), "pr",  vec![0.79, 0.069, 0.14, 0.27, 0.2]);

    let selection_2 = prt.cracker_select_in_three(2, 2, true, true);
    assert_base_i64_column_equals(selection_2.clone(), "src", vec![2]);
    assert_base_i64_column_equals(selection_2.clone(), "dst", vec![1]);
    assert_base_f64_column_equals(selection_2.clone(), "pr",  vec![0.41]);

    let selection_3 = prt.cracker_select_in_three(1, 1, true, true);
    assert_base_i64_column_equals(selection_3.clone(), "src", vec![1, 1, 1]);
    assert_base_i64_column_equals(selection_3.clone(), "dst", vec![3, 3, 5]);
    assert_base_f64_column_equals(selection_3.clone(), "pr",  vec![0.71, 0.9, 0.78]);

    let selection_4 = prt.cracker_select_in_three(3, 3, true, true);
    assert_base_i64_column_equals(selection_4.clone(), "src", vec![3, 3, 3]);
    assert_base_i64_column_equals(selection_4.clone(), "dst", vec![4, 5, 5]);
    assert_base_f64_column_equals(selection_4.clone(), "pr",  vec![0.23, 0.038, 0.64]);
    // After the BFS the cracker column should be fully clustered
    assert_eq!(prt.crk_col.crk, vec![1, 1, 1, 2, 3, 3, 3, 5, 5, 5, 5, 5]);
}

#[test]
fn float_can_exploit_cracker_index_for_selecting_single_value_small_table() {
    let mut prt = pr_table(vec![4,    4,     4,    2,    4,    3],
                           vec![3,    3,     2,    1,    5,    4],
                           vec![0.78, 0.082, 0.51, 0.49, 0.87, 0.64]);

    let selection_1 = prt.cracker_select_in_three(3, 3, true, true);
    assert_base_i64_column_equals(selection_1.clone(), "src", vec![3]);
    assert_base_i64_column_equals(selection_1.clone(), "dst", vec![4]);
    assert_base_f64_column_equals(selection_1.clone(), "pr",  vec![0.64]);

    let selection_2 = prt.cracker_select_in_three(4, 4, true, true);
    assert_base_i64_column_equals(selection_2.clone(), "src", vec![4, 4, 4, 4]);
    assert_base_i64_column_equals(selection_2.clone(), "dst", vec![2, 3, 5, 3]);
    assert_base_f64_column_equals(selection_2.clone(), "pr",  vec![0.51, 0.082, 0.87, 0.78]);

    let selection_3 = prt.cracker_select_in_three(2, 2, true, true);
    assert_base_i64_column_equals(selection_3.clone(), "src", vec![2]);
    assert_base_i64_column_equals(selection_3.clone(), "dst", vec![1]);
    assert_base_f64_column_equals(selection_3.clone(), "pr",  vec![0.49]);

    let selection_4 = prt.cracker_select_in_three(5, 5, true, true);
    assert_base_i64_column_equals(selection_4.clone(), "src", vec![]);
    assert_base_i64_column_equals(selection_4.clone(), "dst", vec![]);
    assert_base_f64_column_equals(selection_4.clone(), "pr",  vec![]);
    // After the BFS the cracker column should be fully clustered
    assert_eq!(prt.crk_col.crk, vec![2, 3, 4, 4, 4, 4]);
}

#[test]
fn float_repeat_queries_return_same_results() {
    let mut prt = pr_table(vec![3,    1,    5,    5,    1,    5,    2,    3,    1,     5,    5,    3],
                           vec![5,    3,    2,    1,    5,    1,    1,    4,    3,     1,    2,    5],
                           vec![0.91, 0.98, 0.31, 0.37, 0.96, 0.41, 0.63, 0.58, 0.009, 0.14, 0.77, 0.3]);

    let selection_1 = prt.cracker_select_specific(5);
    assert_base_i64_column_equals(selection_1.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_i64_column_equals(selection_1.clone(), "dst", vec![2, 1, 1, 2, 1]);
    assert_base_f64_column_equals(selection_1.clone(), "pr",  vec![0.31, 0.41, 0.14, 0.77, 0.37]);

    let selection_2 = prt.cracker_select_specific(5);
    assert_base_i64_column_equals(selection_2.clone(), "src", vec![5, 5, 5, 5, 5]);
    assert_base_i64_column_equals(selection_2.clone(), "dst", vec![2, 1, 1, 2, 1]);
    assert_base_f64_column_equals(selection_1.clone(), "pr",  vec![0.31, 0.41, 0.14, 0.77, 0.37]);
}

#[test]
fn can_rearrange_tuples_in_multityped_table() {
    let mut table = pr_table(vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![0.1, 0.2, 0.3, 0.4]);
    table.rearrange(vec![2, 1, 3, 0].iter());
    assert_base_i64_column_equals(table.clone(), "src", vec![3, 2, 4, 1]);
    assert_base_i64_column_equals(table.clone(), "dst", vec![7, 6, 8, 5]);
    assert_base_f64_column_equals(table.clone(), "pr", vec![0.3, 0.2, 0.4, 0.1]);
}