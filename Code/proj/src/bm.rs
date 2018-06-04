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

fn main() {
    load_person_csv::test();
}