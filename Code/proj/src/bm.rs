extern crate time;
extern crate rand;
extern crate bit_vec;
extern crate csv;
#[macro_use]
extern crate serde_derive;
extern crate lib;

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
pub mod personrank;

fn main() {
    println!("BFS small check");
    bfs::example_test_bfs_methods();
    println!();
    println!("PR small check");
    pagerank::example_test_pagerank_methods();
    println!();
    println!("PR S3G2 check");
    check_s3g2_pr();
}

fn prep_graphviz(src: Vec<i64>, dst: Vec<i64>) {
    print!("digraph {} ", "{");
    for i in 0..src.len() {
        print!("{} -> {}; ", src[i], dst[i]);
    }
    print!("{}\n", "}");
}

fn check_s3g2_pr() {
    let decracked_ranks = personrank::decracked_personrank(1);
    let reco_ranks      = personrank::reco_personrank(1);
    let coco_ranks      = personrank::coco_personrank(1);
    let underswap_ranks = personrank::underswap_personrank(1);
    let overswap_ranks  = personrank::overswap_personrank(1);

    for (k, v) in &decracked_ranks {
        assert!((*v - reco_ranks[k]).abs() < 0.0005);
        assert!((*v - coco_ranks[k]).abs() < 0.0005);
        assert!((*v - underswap_ranks[k]).abs() < 0.0005);
        assert!((*v - overswap_ranks[k]).abs() < 0.0005);
    }
}
