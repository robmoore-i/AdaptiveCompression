use decomposed_cracking;
use recognitive_compression;
use compactive_compression;
use underswap_rle_compression;
use overswap_rle_compression;

use load_person_csv;

use std::collections::HashMap;
use time::PreciseTime;

fn get_out_degree(vertices: &Vec<i64>, src: &Vec<i64>) -> HashMap<i64, i64> {
    let mut out_degree: HashMap<i64, i64> = HashMap::new();
    for v in vertices {
        out_degree.insert(*v, 0);
    }
    for v in src {
        match out_degree.get_mut(&v) {
            Some(d) => { *d += 1; },
            None    => {},
        }
    }
    out_degree
}

pub fn sf1_decracked_personrank() {
    // Setup

    let start = PreciseTime::now();
    let people= load_person_csv::sf1_nodes();
    let (src, dst) = load_person_csv::sf1_edges_adjl();
    let vertices= people.iter().map(|p|p.id).collect();
    let out_degree = get_out_degree(&vertices, &src);
    let n = people.len();
    let mut adjacency_list = decomposed_cracking::from_adjacency_vectors(src, dst, "dst");
    let d = 0.85;
    let max_iterations = 10;
    let end = PreciseTime::now();

    println!("setup time = {:?}", (start.to(end)).to_string());

    // Personrank

    let m = (1.0 - d) / ((n as i64) as f64);

    let mut rank: HashMap<i64, f64> = HashMap::new();
    let mut new_rank: HashMap<i64, f64> = HashMap::new();

    let initial_rank = (n as f64).recip();
    for v in &vertices {
        rank.insert(*v, initial_rank);
        new_rank.insert(*v, initial_rank);
    }

    let mut iterations = 0;
    loop {
        for v in &vertices {
            let mut inherited_rank = 0.0;

            for w in adjacency_list.cracker_select_specific(*v).get_col("src".to_string()).unwrap().v.iter() {
                inherited_rank += rank[&w] / (out_degree[&w] as f64);
            }

            new_rank.insert(*v, m + d * inherited_rank);
        }

        rank = new_rank.clone();

        iterations += 1;
        if iterations > max_iterations {
            break;
        }
    }
}
