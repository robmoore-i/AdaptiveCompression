extern crate time;
extern crate rand;
extern crate bit_vec;

mod utils;

mod cracker_index;
mod column;
mod decomposed_cracking;
mod recognitive_compression;
mod compactive_compression;
mod underswap_rle_compression;

mod datagen;
mod bfs;
mod pagerank;

mod load_person_csv;

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
