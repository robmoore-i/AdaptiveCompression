use decomposed_cracking;
use recognitive_compression;
use compactive_compression;
use underswap_rle_compression;
use overswap_rle_compression;

use load_person_csv;

use std::collections::HashMap;
use time::PreciseTime;
use time::Duration;

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

pub fn benchmark_all(scale_factor: i16, pagerank_iterations: i16, averaging_iterations: usize) {
    let (people, (src, dst)) = match scale_factor {
        1 => (load_person_csv::sf1_nodes(), load_person_csv::sf1_edges_adjl()),
        3 => (load_person_csv::sf3_nodes(), load_person_csv::sf3_edges_adjl()),
        10 => (load_person_csv::sf10_nodes(), load_person_csv::sf10_edges_adjl()),
        _ => panic!("No data for scale_factor: {}", scale_factor),
    };
    let vertices = get_vertices(&people);
    
    let mut preclustered_times = Vec::new();
    let mut decracked_times = Vec::new();
    let mut reco_times = Vec::new();
    let mut coco_times = Vec::new();
    let mut underswap_times = Vec::new();
    let mut overswap_times = Vec::new();

    for i in 0..averaging_iterations {
        let start = PreciseTime::now();
        // let unoptimised_ranks = unoptimised_personrank(vertices.clone(), src.clone(), dst.clone(), max_iterations);
        let (preclustered_ranks, preclustered_t) = preclustered_personrank(vertices.clone(), src.clone(), dst.clone(), pagerank_iterations);
        let (decracked_ranks, decracked_t)       = decracked_personrank(vertices.clone(), src.clone(), dst.clone(), pagerank_iterations);
        let (reco_ranks, reco_t)                 = reco_personrank(vertices.clone(), src.clone(), dst.clone(), pagerank_iterations);
        let (coco_ranks, coco_t)                 = coco_personrank(vertices.clone(), src.clone(), dst.clone(), pagerank_iterations);
        let (underswap_ranks, underswap_t)       = underswap_personrank(vertices.clone(), src.clone(), dst.clone(), pagerank_iterations);
        let (overswap_ranks, overswap_t)         = overswap_personrank(vertices.clone(), src.clone(), dst.clone(), pagerank_iterations);

        preclustered_times.push(preclustered_t);
        decracked_times.push(decracked_t);
        reco_times.push(reco_t);
        coco_times.push(coco_t);
        underswap_times.push(underswap_t);
        overswap_times.push(overswap_t);
        
        let epsilon = 0.00001;

        for (k, v) in &preclustered_ranks {
            assert!((*v - decracked_ranks[k]).abs() < epsilon);
            assert!((*v - reco_ranks[k]).abs() < epsilon);
            assert!((*v - coco_ranks[k]).abs() < epsilon);
            assert!((*v - underswap_ranks[k]).abs() < epsilon);
            assert!((*v - overswap_ranks[k]).abs() < epsilon);
        }

        println!("Iteration {} done in {}", i, start.to(PreciseTime::now()));
    }

    println!("Preclustered: {}", preclustered_times.iter().fold(Duration::hours(0), |sum, val| sum + *val) / (averaging_iterations as i32));
    println!("Decracked:    {}", decracked_times.iter().fold(Duration::hours(0), |sum, val| sum + *val) / (averaging_iterations as i32));
    println!("Reco:         {}", reco_times.iter().fold(Duration::hours(0), |sum, val| sum + *val) / (averaging_iterations as i32));
    println!("Coco:         {}", coco_times.iter().fold(Duration::hours(0), |sum, val| sum + *val) / (averaging_iterations as i32));
    println!("Underswap:    {}", underswap_times.iter().fold(Duration::hours(0), |sum, val| sum + *val) / (averaging_iterations as i32));
    println!("Overswap:     {}", overswap_times.iter().fold(Duration::hours(0), |sum, val| sum + *val) / (averaging_iterations as i32));
}

pub fn unoptimised_personrank(vertices: Vec<i64>, src: Vec<i64>, dst: Vec<i64>, max_iterations: i16) -> HashMap<i64, f64> {
    println!("unoptimised personrank");
    // Setup

    let start = PreciseTime::now();

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

pub fn preclustered_personrank(vertices: Vec<i64>, mut src: Vec<i64>, mut dst: Vec<i64>, max_iterations: i16) -> (HashMap<i64, f64>, Duration) {
    println!("preclustered personrank");

    let start = PreciseTime::now();

    let out_degree = get_out_degree(&vertices, &src);
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();

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
    let alg_time = setup_end.to(alg_end);

    (new_rank, alg_time)
}

pub fn decracked_personrank(vertices: Vec<i64>, src: Vec<i64>, dst: Vec<i64>, max_iterations: i16) -> (HashMap<i64, f64>, Duration) {
    println!("decracked personrank");

    let start = PreciseTime::now();

    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = decomposed_cracking::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();

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
    let alg_time = setup_end.to(alg_end);

    (new_rank, alg_time)
}

pub fn reco_personrank(vertices: Vec<i64>, src: Vec<i64>, dst: Vec<i64>, max_iterations: i16) -> (HashMap<i64, f64>, Duration) {
    println!("reco personrank");

    let start = PreciseTime::now();

    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = recognitive_compression::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();

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
    let alg_time = setup_end.to(alg_end);

    (new_rank, alg_time)
}

pub fn coco_personrank(vertices: Vec<i64>, src: Vec<i64>, dst: Vec<i64>, max_iterations: i16) -> (HashMap<i64, f64>, Duration) {
    println!("coco personrank");

    let start = PreciseTime::now();

    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = compactive_compression::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();

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
    let alg_time = setup_end.to(alg_end);

    (new_rank, alg_time)
}

pub fn underswap_personrank(vertices: Vec<i64>, src: Vec<i64>, dst: Vec<i64>, max_iterations: i16) -> (HashMap<i64, f64>, Duration) {
    println!("underswap personrank");

    let start = PreciseTime::now();

    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = underswap_rle_compression::from_adjacency_vectors(src, dst, "dst");

    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();

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
    let alg_time = setup_end.to(alg_end);

    (new_rank, alg_time)
}

pub fn overswap_personrank(vertices: Vec<i64>, src: Vec<i64>, dst: Vec<i64>, max_iterations: i16) -> (HashMap<i64, f64>, Duration) {
    println!("overswap personrank");

    let start = PreciseTime::now();
    
    let out_degree = get_out_degree(&vertices, &src);
    let mut adjacency_list = overswap_rle_compression::from_adjacency_vectors(src, dst, "dst");
    let (n, d) = (vertices.len(), 0.85);

    let setup_end = PreciseTime::now();

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
    let alg_time = setup_end.to(alg_end);

    (new_rank, alg_time)
}