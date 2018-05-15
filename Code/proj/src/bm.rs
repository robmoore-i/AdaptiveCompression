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

fn main() {
    bfs::run();
}

fn print_tests() {
    println!("=== BFS ===");
    bfs::test_bfs_methods();
    println!("\n=== PR ===");
    pagerank::test_pagerank_methods();
}
