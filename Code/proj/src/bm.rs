extern crate time;
extern crate rand;
extern crate bit_vec;

mod utils;

mod cracker_index;
mod column;
mod decomposed_cracking;
mod recognitive_compression;
mod compactive_compression;
mod intrafragment_compression;

mod datagen;
mod bfs;
mod pagerank;

fn main() {
    println!("=== BFS ===");
    bfs::test_bfs_methods();
    println!("\n=== PR ===");
    pagerank::test_pagerank_methods();
}