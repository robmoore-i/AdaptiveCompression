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

use time::PreciseTime;

fn main() {
    check_s3g2_pr();
    let sf = 1;
    let mi = 10;
//    personrank::decracked_personrank(sf, mi);
//    personrank::reco_personrank(sf, mi);
//    personrank::coco_personrank(sf, mi);
//    personrank::underswap_personrank(sf, mi);
//    personrank::overswap_personrank(sf, mi);
}

fn prep_graphviz(src: Vec<i64>, dst: Vec<i64>) {
    print!("digraph {} ", "{");
    for i in 0..src.len() {
        print!("{} -> {}; ", src[i], dst[i]);
    }
    print!("{}\n", "}");
}

fn check_s3g2_pr() {
    let sf = 1;
    let mi = 10;
    let decracked_ranks = personrank::decracked_personrank(sf, mi);
    let reco_ranks      = personrank::reco_personrank(sf, mi);
    let coco_ranks      = personrank::coco_personrank(sf, mi);
    let underswap_ranks = personrank::underswap_personrank(sf, mi);
    let overswap_ranks  = personrank::overswap_personrank(sf, mi);

    for (k, v) in &decracked_ranks {
        assert!((*v - reco_ranks[k]).abs() < 0.0005);
        assert!((*v - coco_ranks[k]).abs() < 0.0005);
        assert!((*v - underswap_ranks[k]).abs() < 0.0005);
        assert!((*v - overswap_ranks[k]).abs() < 0.0005);
    }
}
