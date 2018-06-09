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

fn get_vertices(people: &Vec<load_person_csv::Person>) -> Vec<i64> {
    people.iter().map(|p|p.id).collect()
}

pub fn unoptimised_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("unoptimised personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

    let m = (1.0 - d) / (n as f64);

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

            for i in 0..src.len() {
                if dst[i] == *v {
                    let w = src[i];
                    inherited_rank += rank[&w] / (out_degree[&w] as f64);
                }
            }

            new_rank.insert(*v, m + d * inherited_rank);
        }

        rank = new_rank.clone();

        iterations += 1;
        if iterations > max_iterations {
            break;
        }
    }

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}

pub fn preclustered_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("preclustered personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (mut src, mut dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

    let e = src.len();

    // Cluster by dst column.
    let mut row_store = Vec::with_capacity(e);
    for i in 0..e {
        row_store.push((src[i], dst[i]));
    }
    row_store.sort_by_key(|&k| k.1);
    for i in 0..e {
        src[i] = row_store[i].0;
        dst[i] = row_store[i].1;
    }

    let m = (1.0 - d) / (n as f64);

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

            let i;
            match dst.binary_search(v) {
                Ok(x)  => i = x,
                Err(_) => {
                    new_rank.insert(*v, m);
                    continue;
                },
            }

            let mut inc_idx = i.clone();
            let mut dec_idx = i.clone();

            loop {
                let w = src[inc_idx];
                inc_idx += 1;
                inherited_rank += rank[&w] / (out_degree[&w] as f64);
                if inc_idx >= e {
                    break;
                } else if dst[inc_idx] != *v {
                    break;
                }
            }
            while dec_idx > 0 {
                dec_idx -= 1;
                if dst[dec_idx] != *v {
                    break;
                }
                let w = src[dec_idx];
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

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}

pub fn decracked_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("decracked personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = decomposed_cracking::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

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

            for w in adjacency_list.cracker_select_specific(*v, "src") {
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

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}

pub fn reco_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("reco personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

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

            for w in adjacency_list.cracker_select_specific(*v, "src") {
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

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}

pub fn coco_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("coco personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = compactive_compression::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

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

            for w in adjacency_list.cracker_select_specific(*v, "src") {
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

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}

pub fn underswap_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("underswap personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = underswap_rle_compression::from_adjacency_vectors(src, dst, "dst");

    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

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

            for w in adjacency_list.cracker_select_specific(*v, "src") {
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

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}

pub fn overswap_personrank(sf: i16, max_iterations: i16) -> HashMap<i64, f64> {
    println!("overswap personrank, sf = {}, iterations = {}", sf, max_iterations);
    // Setup

    let start = PreciseTime::now();

    let (people, (src, dst)) = match sf {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for sf: {}", sf),
    };

    let vertices = get_vertices(&people);
    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = overswap_rle_compression::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();
    println!("cfg_time = {:?}", (start.to(setup_end)).to_string());

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

            for w in adjacency_list.cracker_select_specific(*v, "src") {
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

    let alg_end = PreciseTime::now();
    println!("run_time = {:?}", (setup_end.to(alg_end)).to_string());

    new_rank
}