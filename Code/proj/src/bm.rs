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
    personrank::benchmark_all(10, 50, 10);
}

// Gets for each method the average over (i) runs of the break-even point on a random tree of size (n).
fn break_even_points(n: i64, i: usize) {
    let mut decracked_queries = Vec::new();
    let mut reco_queries = Vec::new();
    let mut coco_queries = Vec::new();
    let mut underswap_queries = Vec::new();
    let mut overswap_queries = Vec::new();

    for j in 0..i {
        let start = PreciseTime::now();
        let (src, dst) = datagen::randomly_connected_tree(n);
        let start_node = rand::thread_rng().gen_range(1, n);
        println!("Created tree {} after {} seconds", j, start.to(PreciseTime::now()).to_string());

        let start = PreciseTime::now();
        bfs::precluster(&src, &dst);
        let d = start.to(PreciseTime::now());

        decracked_queries.push(bfs::decracked_bfs_adjl_until(decomposed_cracking::from_adjacency_vectors(src.clone(), dst.clone(), "src"), start_node, d));
        reco_queries.push(bfs::reco_bfs_adjl_until(recognitive_compression::from_adjacency_vectors(src.clone(), dst.clone(), "src"), start_node, d));
        coco_queries.push(bfs::coco_bfs_adjl_until(compactive_compression::from_adjacency_vectors(src.clone(), dst.clone(), "src"), start_node, d));
        underswap_queries.push(bfs::underswap_bfs_adjl_until(underswap_rle_compression::from_adjacency_vectors(src.clone(), dst.clone(), "src"), start_node, d));
        overswap_queries.push(bfs::overswap_bfs_adjl_until(overswap_rle_compression::from_adjacency_vectors(src.clone(), dst.clone(), "src"), start_node, d));
    }

    println!("Decracked: {}", decracked_queries.iter().fold(0 as f64, |sum, val| sum + (*val  as f64)) / (i as f64));
    println!("Reco:      {}", reco_queries.iter().fold(0 as f64, |sum, val| sum + (*val  as f64)) / (i as f64));
    println!("Coco:      {}", coco_queries.iter().fold(0 as f64, |sum, val| sum + (*val  as f64)) / (i as f64));
    println!("Underswap: {}", underswap_queries.iter().fold(0 as f64, |sum, val| sum + (*val  as f64)) / (i as f64));
    println!("Overswap:  {}", overswap_queries.iter().fold(0 as f64, |sum, val| sum + (*val  as f64)) / (i as f64));
}

fn prep_graphviz(src: Vec<i64>, dst: Vec<i64>) {
    print!("digraph {} ", "{");
    for i in 0..src.len() {
        print!("{} -> {}; ", src[i], dst[i]);
    }
    print!("{}\n", "}");
}

fn speed_test(sf: i16, mi: i16, n: i8) {
    println!("Speed test over {} iterations", n);
    let mut diffs: Vec<Duration> = Vec::new();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        3 => (load_person_csv::sf3_nodes(), load_person_csv::sf3_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for scale_factor: {}", sf),
    };
    let vertices: Vec<i64> = people.iter().map(|p|p.id).collect();

    for _ in 0..n {
        let (preclustered_ranks, _) = personrank::preclustered_personrank(vertices.clone(), src.clone(), dst.clone(), mi);
        let (decracked_ranks, dt) = personrank::decracked_personrank(vertices.clone(), src.clone(), dst.clone(), mi);
        let (reco_ranks, ut) = personrank::reco_personrank(vertices.clone(), src.clone(), dst.clone(), mi);

        let diff = dt - ut;
        diffs.push(diff);

        let epsilon = 0.00001;

        for (k, v) in &preclustered_ranks {
            assert!((*v - decracked_ranks[k]).abs() < epsilon);
            assert!((*v - reco_ranks[k]).abs() < epsilon);
        }
    }

    let sum: Duration = diffs.iter().fold(Duration::hours(0), |sum, val| sum + *val);
    let avg = sum / (diffs.len() as i32);

    println!("===");
    println!("On avg decracked is {:?} slower than reco", avg.to_string());
    println!("===");
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
