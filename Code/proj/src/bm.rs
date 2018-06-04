extern crate time;
extern crate rand;
extern crate bit_vec;
extern crate csv;
#[macro_use]
extern crate serde_derive;

pub mod utils;

pub mod cracker_index;
pub mod column;
pub mod decomposed_cracking;
pub mod recognitive_compression;
pub mod compactive_compression;
pub mod underswap_rle_compression;
pub mod overswap_rle_compression;

pub mod datagen;
pub mod bfs;
pub mod pagerank;

pub mod load_person_csv;

use std::collections::HashMap;

fn main() {
    let people = load_person_csv::small_nodes();
    let (src, dst) = load_person_csv::small_edges_adjl();

    let n = people.len();
    let initial_rank = (n as f64).recip();
    let mut vertices: HashMap<i64, (f64, i64)> = HashMap::new();
    for p in &people {
        vertices.insert(p.id, (initial_rank, 0));
    }
    for v in src {
        match vertices.get_mut(&v) {
            Some((rank, out_degree)) => *out_degree += 1,
            None => {},
        }
    }

    for i in 0..10 {
        let id = people[i].id;
        let data = vertices.get(&id);
        println!("{} has data {:?}", id, data);
    }
}