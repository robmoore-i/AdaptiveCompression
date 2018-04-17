use rand::Rng; // HMM
use rand;

// Finds the directed density of a graph with n nodes and e edges. Returned as a float.
pub fn graph_density(n: i64, e: usize) -> f64 {
    (e as f64) / ((n * (n - 1)) as f64)
}

// Deals out the numbers from 0 to n-1 inclusive in a random order as usizes.
fn deal(n: usize) -> Vec<usize> {
    // Put 1 - n in a bag.
    let mut dealing: Vec<usize> = Vec::with_capacity(n as usize);
    for i in 0..n {
        dealing.push(i);
    }
    let mut dealt: Vec<usize> = Vec::with_capacity(n as usize);
    let mut rng = rand::thread_rng();
    let mut chosen = 0;
    while dealing.len() > 1 {
        let random = rng.gen_range(0, n - chosen);
        dealt.push(dealing[random]);
        dealing.remove(random);
        chosen = chosen + 1;
    }
    dealt.push(dealing[0]);
    dealt
}

fn pairwise_shuffle(src: Vec<i64>, dst: Vec<i64>) -> (Vec<i64>, Vec<i64>) {
    let n = src.len();
    let mut src_shuffled = Vec::with_capacity(n);
    let mut dst_shuffled = Vec::with_capacity(n);
    for i in deal(n) {
        src_shuffled.push(src[i]);
        dst_shuffled.push(dst[i]);
    }
    (src_shuffled, dst_shuffled)
}

// Returns a bidirectionally connected tree for n nodes, which are numbered 1 to n inclusive.
pub fn randomly_connected_tree(n: i64) -> (Vec<i64>, Vec<i64>) {
    let mut add_order: Vec<i64> = deal(n as usize).iter().map(|x| 1 + *x as i64).collect();

    let node_1 = *rand::thread_rng().choose(&add_order).unwrap();
    let mut node_2 = *rand::thread_rng().choose(&add_order).unwrap();
    while node_2 == node_1 {
        node_2 = *rand::thread_rng().choose(&add_order).unwrap();
    }

    let index_1 = add_order.iter().position(|node| *node == node_1).unwrap();
    add_order.remove(index_1);
    let index_2 = add_order.iter().position(|node| *node == node_2).unwrap();
    add_order.remove(index_2);

    let mut src_col = vec![node_1, node_2];
    let mut dst_col = vec![node_2, node_1];
    for src in add_order {
        let dst = *rand::thread_rng().choose(&src_col).unwrap();

        src_col.push(src);
        dst_col.push(dst);

        src_col.push(dst);
        dst_col.push(src);
    }
    pairwise_shuffle(src_col, dst_col)
}

