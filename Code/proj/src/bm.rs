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
pub mod overswap_rle_compression;

pub mod datagen;
pub mod bfs;
pub mod pagerank;

pub mod load_person_csv;

fn main() {
    print_tests();
}

fn print_tests() {
//    let sizes = vec![100, 1000, 2000, 5000, 10000, 20000, 30000];
//    bfs::benchmark_sparse_bfs_csv(sizes);
    bfs::bait();
}
