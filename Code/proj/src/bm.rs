extern crate time;
extern crate rand;
extern crate bit_vec;

pub mod utils;

pub mod cracker_index;
pub mod column;
pub mod decomposed_cracking;
pub mod recognitive_compression;
pub mod compactive_compression;
pub mod underswap_rle_compression;

pub mod datagen;
pub mod bfs;
pub mod pagerank;

pub mod load_person_csv;

fn main() {
    load_person_csv::print_sf10_dirs();
    print_tests();
}

fn print_tests() {
    println!("=== BFS ===");
    bfs::test_bfs_methods();
    println!("\n=== PR ===");
    pagerank::test_pagerank_methods();
}
