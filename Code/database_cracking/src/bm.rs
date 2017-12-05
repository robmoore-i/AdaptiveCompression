extern crate cracking;

use cracking::db::*;
use std::collections::HashMap;
use std::iter;

fn main() {
    let t = fully_connected_graph(3);
    let selection = t.select_in_two("src".to_string(), 4);
    let src = selection.get_col("src".to_string());
    let dst = selection.get_col("dst".to_string());
    match src {
        Some(ref col) => println!("src = {:?}", col.v),
        None          => println!("src failed"),
    }
    match dst {
        Some(ref col) => println!("dst = {:?}", col.v),
        None          => println!("dst failed"),
    }
}

fn fully_connected_graph(n: i64) -> Table {
    let mut t = Table::new();
    t.new_columns(vec!["src".to_string(), "dst".to_string()]);
    for i in 1..(n+1) {
        let mut connections = HashMap::new();
        let i_vec: Vec<i64> = iter::repeat(i as i64).take((n-1) as usize).collect();
        connections.insert("src".to_string(), i_vec);
        let mut dst_nodes: Vec<i64> = Vec::new();
        dst_nodes.extend(1..i);
        dst_nodes.extend((i+1)..(n+1));
        connections.insert("dst".to_string(), dst_nodes);
        t.insert(&mut connections)
    }
    t.set_crk_col("src".to_string());
    t
}

// Given a 2-column ADJACENCY_LIST of src_node!dst_node, this function
// visits every node in the graph from START_NODE.
fn bfs(adjacency_list: Table, start_node: i64) {
}