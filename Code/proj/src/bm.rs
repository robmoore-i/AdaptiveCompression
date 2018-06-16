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
pub mod quicksort;

use time::PreciseTime;
use time::Duration;
use rand::Rng;

fn main() {
    let sf = 1;
    let mi = 10;
}

fn prep_graphviz(src: Vec<i64>, dst: Vec<i64>) {
    print!("digraph {} ", "{");
    for i in 0..src.len() {
        print!("{} -> {}; ", src[i], dst[i]);
    }
    print!("{}\n", "}");
}

fn check_s3g2_pr(sf: i16, mi: i16) {
    let preclustered_ranks = personrank::preclustered_personrank(sf, mi);
    let decracked_ranks    = personrank::decracked_personrank(sf, mi);
    let reco_ranks         = personrank::reco_personrank(sf, mi);
    let coco_ranks         = personrank::coco_personrank(sf, mi);
    let underswap_ranks    = personrank::underswap_personrank(sf, mi);
    let overswap_ranks     = personrank::overswap_personrank(sf, mi);

    let epsilon = 0.00001;
    
    for (k, v) in &preclustered_ranks {
        assert!((*v - decracked_ranks[k]).abs() < epsilon);
        assert!((*v - reco_ranks[k]).abs() < epsilon);
        assert!((*v - coco_ranks[k]).abs() < epsilon);
        assert!((*v - underswap_ranks[k]).abs() < epsilon);
        assert!((*v - overswap_ranks[k]).abs() < epsilon);
    }
}

fn speed_test(sf: i16, mi: i16, n: i8) {
    println!("Speed test over {} iterations", n);
    let mut diffs: Vec<Duration> = Vec::new();

    for _ in 0..n {
        let ds = PreciseTime::now();
        let decracked_ranks = personrank::decracked_personrank(sf, mi);
        let de = PreciseTime::now();

        let dt = ds.to(de);

        let s = PreciseTime::now();
        let underswap_ranks = personrank::underswap_personrank(sf, mi);
        let e = PreciseTime::now();

        let t = s.to(e);

        let diff = dt - t;
        diffs.push(diff);

        let epsilon = 0.00001;

        for (k, v) in &decracked_ranks {
            assert!((*v - underswap_ranks[k]).abs() < epsilon);
        }
    }

    let sum: Duration = diffs.iter().fold(Duration::hours(0), |sum, val| sum + *val);
    let avg = sum / (diffs.len() as i32);

    println!("===");
    println!("Avg diff: {:?}", avg.to_string());
    println!("===");
}

fn run_all(sf: i16, mi: i16) {
    personrank::decracked_personrank(sf, mi);
    personrank::reco_personrank(sf, mi);
    personrank::coco_personrank(sf, mi);
    personrank::underswap_personrank(sf, mi);
    personrank::overswap_personrank(sf, mi);
}

fn speed_test_tighten(selectivity: f64, n: usize) {
    let mut rng = rand::thread_rng();
    let p_low = 0;

    let mut crk = Vec::new();
    let mut run_lengths = Vec::new();
    let count = 1000000;
    let p_high = count;

    let v = selectivity.recip().ceil() as i64;
    for _ in 0..count {
        let random = rng.gen_range(0, v);
        crk.push(random);
        run_lengths.push(1);
    }

    let x = 1;
    print!("Selecting {} from {}, {} times: ", x, v, n);

    let mut ts: Vec<Duration> = Vec::new();
    for _ in 0..n {
        let t = time_low_tighten_1(crk.clone(), run_lengths.clone(), count, p_low, p_high, x);
        ts.push(t);
    }

    let sum: Duration = ts.iter().fold(Duration::hours(0), |sum, val| sum + *val);
    let avg = sum / (ts.len() as i32);
    println!("Average time {} to tighten over {} values at {} selectivity on the {} side", avg, count, selectivity, "low");
}

fn time_low_tighten_1(crk: Vec<i64>, mut run_lengths: Vec<usize>, count: usize, mut p_low: usize, p_high: usize, x: i64) -> Duration {
    let start = PreciseTime::now();

    while crk[p_low] < x && p_low < p_high {
        let mut rl = run_lengths[p_low];

        while p_low + rl >= count && p_low + 1 < count { // Evade overflow.
            p_low += 1;
            rl = run_lengths[p_low];
        }

        if p_low + rl >= count {
            break;
        }

        if crk[p_low + rl] == crk[p_low] {
            while crk[p_low + rl] == crk[p_low] {
                let inc = run_lengths[p_low + rl];
                if p_low + rl + inc >= p_high {
                    break;
                }
                rl += inc;
            }
            run_lengths[p_low]          = rl;
            run_lengths[p_low + rl - 1] = rl;
        }
        p_low += rl;
    }

    let end = PreciseTime::now();
    start.to(end)
}