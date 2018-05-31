// Decomposed cracking
// Overswap RLE, recognitive compression

// uses map! macro.

use column::Column;
use column::IntCol;
use cracker_index::AVLCrackerIndex;
use std::collections::HashMap;
use std::slice::Iter;

#[derive(Clone)]
pub struct OverswapRLETable {
    pub count: usize,
    pub crk_col: IntCol,
    pub columns: HashMap<String, IntCol>,
    pub dbg_switch: bool,
}

impl OverswapRLETable {
    pub fn new() -> OverswapRLETable {
        OverswapRLETable {
            count: 0,
            crk_col: IntCol::empty(),
            columns: HashMap::new(),
            dbg_switch: false,
        }
    }

    pub fn print_cols(&self) {
        println!("crk: {:?}", self.crk_col.crk);
        println!("ofs: {:?}", self.crk_col.ofs);
        println!("rls: {:?}", self.crk_col.run_lengths);
        for (name, col) in self.columns.clone() {
            println!("{}: {:?}", name, col.v);
        }
    }

    pub fn print_rl_crk(&self) {
        println!("crk: {:?}", self.crk_col.crk);
        println!("rls: {:?}", self.crk_col.run_lengths);
    }

    pub fn print_rl_crk_range(&self, lower: usize, upper: usize) {
        print!("crk: [{}", self.crk_col.crk[lower]);
        for i in (lower + 1)..upper {
            print!(", {}", self.crk_col.crk[i])
        }
        print!("]\n");
        print!("rls: [{}", self.crk_col.run_lengths[lower]);
        for i in (lower + 1)..upper {
            print!(", {}", self.crk_col.run_lengths[i])
        }
        print!("]\n");
    }

    pub fn print_crk(&self) {
        println!("crk: {:?}", self.crk_col.crk);
    }

    pub fn new_columns(&mut self, col_names: Vec<&str>) {
        for col in col_names {
            self.columns.insert(col.to_string(), IntCol::empty());
        }
    }

    pub fn set_crk_col(&mut self, col_name: &str) {
        let col = match self.columns.get(&(col_name.to_string())) {
            Some(ref c) => *c,
            None => panic!("set_crk_col: no such col"),
        };
        self.crk_col = col.clone();
    }

    // TODO: Improve exception handling in this function
    pub fn insert(&mut self, new_values: &mut HashMap<&str, Vec<i64>>) {
        let mut n_new_tuples = 0;
        for (key, val) in self.columns.iter_mut() {
            let new_elements = new_values.get_mut(&*(key.clone())).unwrap();
            let n = new_elements.len();
            if n_new_tuples == 0 || n_new_tuples == n {
                val.v.append(new_elements);
                n_new_tuples = n;
            } else {
                panic!("insert: new_values has vectors of differing lengths");
            }
        }
        self.count += n_new_tuples;
    }

    pub fn get_col(&self, col: &str) -> &IntCol {
        self.columns.get(&(col.to_string())).unwrap()
    }

    pub fn get_indices(&self, indices: Iter<usize>) -> OverswapRLETable {
        let mut selection: HashMap<String, IntCol> = HashMap::new();
        for (name, col) in &self.columns {
            let mut v_buffer = Vec::with_capacity(indices.len());
            for &i in indices.clone() {
                v_buffer.push(col.v[i]);
            }
            let mut c_buffer = IntCol::empty();
            c_buffer.v = v_buffer;
            selection.insert(name.clone(), c_buffer);
        }

        let mut t = OverswapRLETable::new();
        t.columns = selection;
        t.count = indices.len();

        let mut indexed_crk_v = Vec::with_capacity(indices.len());
        if self.crk_col.crk.len() > 0 {
            let mut indexed_crk_col = Vec::with_capacity(indices.len());
            for &i in indices.clone() {
                indexed_crk_col.push(self.crk_col.crk[i]);
                indexed_crk_v.push(self.crk_col.v[i]);
            }
            t.crk_col.crk = indexed_crk_col;
            t.crk_col.crk_idx = AVLCrackerIndex::new();
        } else {
            for &i in indices.clone() {
                indexed_crk_v.push(self.crk_col.v[i]);
            }
        }
        t.crk_col.v = indexed_crk_v;
        t
    }

    pub fn rearrange(&mut self, indices: Iter<usize>) {
        for col in self.columns.values_mut() {
            col.rearrange(indices.clone());
        }
        self.crk_col.rearrange(indices.clone());
    }

    // Returns the elements of T where the cracker columns's value equals X
    pub fn cracker_select_specific(&mut self, x: i64) -> OverswapRLETable {
        println!("Selecting {}", x);

        // Init
        if self.crk_col.crk.len() == 0 {
            self.crk_col.crk = self.crk_col.v.clone();
            self.crk_col.base_idx = (0..self.count).collect();
            self.crk_col.run_lengths = vec![1;self.count];
        }

        // Setup
        let mut p_low  = self.crk_col.crk_idx.lower_bound(&x).unwrap_or(0);
        if p_low == self.count {
            return self.get_indices(self.crk_col.base_idx[0..0].iter());
        }
        let mut p_high = self.crk_col.crk_idx.upper_bound(&(x + 1)).unwrap_or(self.count) - 1;

        // Tighten
        while self.crk_col.crk[p_low] < x && p_low < p_high {
            let mut rl = self.crk_col.run_lengths[p_low];

            while p_low + rl >= self.count && p_low + 1 < self.count { // Evade overflow.
                p_low += 1;
                rl = self.crk_col.run_lengths[p_low];
            }
            if p_low + rl >= self.count {
                break;
            }

            if self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                while self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                    let inc = self.crk_col.run_lengths[p_low + rl];
                    if p_low + rl + inc >= p_high {
                        break;
                    }
                    rl += inc;
                }
                self.crk_col.run_lengths[p_low]          = rl;
                self.crk_col.run_lengths[p_low + rl - 1] = rl;
            }
            p_low += rl;
        }

        while self.crk_col.crk[p_high] > x && p_high > p_low {
            let mut rl = self.crk_col.run_lengths[p_high];
            if self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                while self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                    let inc = self.crk_col.run_lengths[p_high - rl];
                    if p_high < rl + inc {
                        break;
                    } else if p_high - (rl + inc) < p_low {
                        break;
                    }
                    rl += inc;
                }
                self.crk_col.run_lengths[p_high]            = rl;
                self.crk_col.run_lengths[(p_high - rl) + 1] = rl;
            }
            p_high -= rl;
        }

        if p_low >= p_high {
            if self.crk_col.crk[p_low] == x {
                return self.get_indices(self.crk_col.base_idx[p_low..(p_low + 1)].iter())
            } else {
                return self.get_indices(self.crk_col.base_idx[0..0].iter())
            }
        }

        // Scan
        let mut p_itr = p_low.clone();
        while p_itr <= p_high {

            println!("x:{}, L:{}, I:{}, H:{}", x, p_low, p_itr, p_high);
            self.print_rl_crk();
            println!();

            if self.crk_col.crk[p_itr] < x {
                let rl_itr = self.crk_col.run_lengths[p_itr];
                let rl_low = self.crk_col.run_lengths[p_low];
                let pad_size = ((rl_itr as i8)- (rl_low as i8)).abs() as usize;

                if rl_itr > rl_low {
                    // Check for overlap:
                    if p_itr < p_low + rl_itr {
                        // Overlap
                        let overlap_size = (p_low + rl_itr) - p_itr;

                        self.crk_col.run_lengths[p_itr + overlap_size - 1] = self.crk_col.run_lengths[p_itr];
                        self.crk_col.run_lengths[p_itr + overlap_size]     = self.crk_col.run_lengths[p_itr];

                        for i in 0..(rl_itr - overlap_size) {
                            self.crk_col.crk.swap(p_low + i, p_itr + overlap_size + i);
                            self.crk_col.base_idx.swap(p_low + i, p_itr + overlap_size + i);
                            self.crk_col.run_lengths.swap(p_low + i, p_itr + overlap_size + i);
                        }
                    } else {
                        // Combine all the runs between L and I
                        self.crk_col.run_lengths[p_low + rl_itr]     = p_itr - p_low;
                        self.crk_col.run_lengths[p_low + rl_itr - 1] = p_itr - p_low;

                        // Now just swap the whole thing.
                        for i in 0..rl_itr {
                            self.crk_col.crk.swap(p_low + i, p_itr + i);
                            self.crk_col.base_idx.swap(p_low + i, p_itr + i);
                            self.crk_col.run_lengths.swap(p_low + i, p_itr + i);
                        }
                        p_itr += rl_itr;
                    }
                    // Advance L by rl[I]
                    // p_low += rl_itr;
                } else if rl_itr < rl_low {
                    // Check for overlap:
                    if p_itr - pad_size <= p_low + rl_low - 1 {
                        // Overlap
                        // Combine the overlapped intermediate runs between L and I.
                        self.crk_col.run_lengths[p_low + rl_itr]     = p_itr - p_low;
                        self.crk_col.run_lengths[p_low + rl_itr - 1] = p_itr - p_low;
                        for i in 0..rl_itr {
                            self.crk_col.crk.swap(p_low + i, p_itr + i);
                            self.crk_col.base_idx.swap(p_low + i, p_itr + i);
                            self.crk_col.run_lengths.swap(p_low + i, p_itr + i);
                        }
                        p_itr += rl_itr;
                    } else {
                        // No overlap
                        let mut p_pad = p_itr - 1;
                        // Critically tighten the padding pointer
                        while p_pad - self.crk_col.run_lengths[p_pad] >= p_itr - pad_size {
                            p_pad -= self.crk_col.run_lengths[p_pad];
                        }
                        let rl_pad = self.crk_col.run_lengths[p_pad];

                        // If the fit isn't exact, amend the runs
                        if p_pad - rl_pad != p_itr - pad_size - 1 {
                            let rem_size = p_pad - (p_itr - pad_size - 1);
                            if rem_size == 0 { panic!(); }

                            // Fix I - |pad| - 1 to P - rl[P] + 1 (beyond padding)
                            self.crk_col.run_lengths[p_pad - rl_pad + 1] -= rem_size;
                            if self.crk_col.run_lengths[p_pad - rl_pad - 1] == 0 { panic!(); }

                            self.crk_col.run_lengths[p_itr - pad_size - 1] = self.crk_col.run_lengths[p_pad - rl_pad + 1];
                            // Fix P             to P - |rem| + 1 (inside padding)
                            self.crk_col.run_lengths[p_pad] = rem_size;
                            self.crk_col.run_lengths[p_pad - rem_size + 1] = rem_size;
                        }
                        // Move the rl marker for the main section of the low-side run to the end of the section.
                        self.crk_col.run_lengths[p_low + rl_itr] = rl_low;

                        // Do the swaps
                        // Main: Swap L to L + rl[I] - 1                    with I to I + rl[I] - 1
                        for i in 0..rl_itr {
                            self.crk_col.crk.swap(p_low + i, p_itr + i);
                            self.crk_col.base_idx.swap(p_low + i, p_itr + i);
                            self.crk_col.run_lengths.swap(p_low + i, p_itr + i);
                        }
                        // Padding: Swap L + rl[I] to L + rl[L] - 1 with I - |pad| to I - 1
                        for i in 0..pad_size {
                            self.crk_col.crk.swap(p_low + rl_itr + i, p_itr - pad_size + i);
                            self.crk_col.base_idx.swap(p_low + rl_itr + i, p_itr - pad_size + i);
                            self.crk_col.run_lengths.swap(p_low + rl_itr + i, p_itr - pad_size + i);
                        }
                        // Advance L by rl[I]
                        // p_low += rl_itr;
                        // Recede I by |pad|
                        p_itr -= pad_size;
                    }
                } else {
                    // Full, immediate swap
                    for i in 0..rl_itr {
                        self.crk_col.crk.swap(p_low + i, p_itr + i);
                        self.crk_col.base_idx.swap(p_low + i, p_itr + i);
                        self.crk_col.run_lengths.swap(p_low + i, p_itr + i);
                    }

                    // Advance L by rl[I]
                    // p_low += rl_itr;
                }

                // Tighten low
                while self.crk_col.crk[p_low] < x && p_low < p_high {
                    let mut rl = self.crk_col.run_lengths[p_low];

                    while p_low + rl >= self.count && p_low + 1 < self.count { // Evade overflow.
                        p_low += 1;
                        rl = self.crk_col.run_lengths[p_low];
                    }
                    if p_low + rl >= self.count {
                        break;
                    }

                    if self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                        while self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                            let inc = self.crk_col.run_lengths[p_low + rl];
                            if p_low + rl + inc >= p_high {
                                break;
                            }
                            rl += inc;
                        }
                        self.crk_col.run_lengths[p_low]          = rl;
                        self.crk_col.run_lengths[p_low + rl - 1] = rl;
                    }
                    p_low += rl;
                }

                if p_itr < p_low {
                    p_itr = p_low.clone();
                }
            } else if self.crk_col.crk[p_itr] > x {
                let rl_itr = self.crk_col.run_lengths[p_itr];
                let rl_high = self.crk_col.run_lengths[p_high];
                let pad_size = ((rl_itr as i8)- (rl_high as i8)).abs() as usize;

                if rl_itr > rl_high {
                    // Check for overlap:
                    if p_high - pad_size < p_itr + rl_itr {
                        // Overlap
                        let overlap_size = (p_itr + rl_itr) - (p_high - pad_size);
                        // Amend rl marker for out-of-order swap
                        self.crk_col.run_lengths[p_itr + (rl_itr - overlap_size) - 1] = rl_itr;
                        self.crk_col.run_lengths[p_itr + (rl_itr - overlap_size)]     = rl_itr;
                        for i in 0..(rl_itr - overlap_size) {
                            self.crk_col.crk.swap(p_itr + i, p_high - rl_itr + 1 + overlap_size + i);
                            self.crk_col.base_idx.swap(p_itr + i, p_high - rl_itr + 1 + overlap_size + i);
                            self.crk_col.run_lengths.swap(p_itr + i, p_high - rl_itr + 1 + overlap_size + i);
                        }
                    } else {
                        // No overlap
                        let mut p_pad = p_high - rl_high;
                        // Critically tighten the padding pointer
                        while p_high - (p_pad - self.crk_col.run_lengths[p_pad]) < rl_itr {
                            p_pad -= self.crk_col.run_lengths[p_pad];
                        }
                        let rl_pad = self.crk_col.run_lengths[p_pad];
                        let rem_size = p_pad - (p_high - rl_itr);
                        if rem_size == 0 { self.print_rl_crk_range(p_itr, p_high + 1);panic!(); }

                        // If the fit isn't exact, amend the runs
                        if p_pad - rl_pad != p_high - rl_itr {
                            // Fix H - rl[I] to P - rl[p] + 1
                            self.crk_col.run_lengths[p_pad - rl_pad + 1] -= rem_size;
                            if self.crk_col.run_lengths[p_pad - rl_pad - 1] == 0 { self.print_rl_crk_range(p_itr, p_high + 1);panic!(); }

                            self.crk_col.run_lengths[p_high - rl_itr] = self.crk_col.run_lengths[p_pad - rl_pad + 1];
                            // Fix P to P - |rem| + 1 (inside padding)
                            self.crk_col.run_lengths[p_pad] = rem_size;
                            self.crk_col.run_lengths[p_pad - rem_size + 1] = rem_size;
                        }
                        // Move the rl marker for the main section of the itr-side run to the end of the section.
                        self.crk_col.run_lengths[p_itr + rl_high] = self.crk_col.run_lengths[p_itr];
                        // Do the swaps
                        // Main: Swap I to I + rl[H] - 1 with H to H - rl[H] + 1
                        for i in 0..rl_high {
                            self.crk_col.crk.swap(p_itr + i, p_high - rl_high + 1 + i);
                            self.crk_col.base_idx.swap(p_itr + i, p_high - rl_high + 1 + i);
                            self.crk_col.run_lengths.swap(p_itr + i, p_high - rl_high + 1 + i);
                        }
                        // Padding: Swap I + rl[H] to I + rl[I] - 1 with H - rl[H] to H - rl[I] + 1
                        for i in 0..pad_size {
                            self.crk_col.crk.swap(p_itr + rl_high + i, p_high - rl_itr + 1 + i);
                            self.crk_col.base_idx.swap(p_itr + rl_high + i, p_high - rl_itr + 1 + i);
                            self.crk_col.run_lengths.swap(p_itr + rl_high + i, p_high - rl_itr + 1 + i);
                        }
                    }
                    // Tighten H by rl[I]
                    // p_high -= rl_itr;
                } else if rl_high > rl_itr {
                    // Check for overlap:
                    if p_high - rl_high + 1 < p_itr + rl_high {
                        // Overlap
                        let overlap_size = (p_itr + rl_high) - (p_high - rl_high + 1);
                        // Amend rl marker for out-of-order swap
                        self.crk_col.run_lengths[p_high - (rl_high - overlap_size) + 1] = rl_high;
                        self.crk_col.run_lengths[p_high - (rl_high - overlap_size)]     = rl_high;
                        for i in 0..(rl_high - overlap_size) {
                            self.crk_col.crk.swap(p_itr + i, p_high - (rl_high - overlap_size) + 1 + i);
                            self.crk_col.base_idx.swap(p_itr + i, p_high - (rl_high - overlap_size) + 1 + i);
                            self.crk_col.run_lengths.swap(p_itr + i, p_high - (rl_high - overlap_size) + 1 + i);
                        }
                    } else {
                        // No overlap
                        let mut p_pad = p_itr + rl_itr;
                        // Critically tighten the padding pointer
                        while p_pad + self.crk_col.run_lengths[p_pad] < p_itr + rl_high {
                            p_pad += self.crk_col.run_lengths[p_pad];
                        }
                        let rl_pad = self.crk_col.run_lengths[p_pad];
                        let rem_size = (p_itr + rl_high) - p_pad;
                        if rem_size == 0 { self.print_rl_crk_range(p_itr, p_high + 1);panic!(); }

                        // If the fit isn't exact, amend the runs
                        if p_pad + rl_pad != p_itr + rl_high {
                            // Fix I + rl[H] to P + rl[P] - 1 (beyond padding)
                            self.crk_col.run_lengths[p_pad + rl_pad - 1] -= rem_size;
                            if self.crk_col.run_lengths[p_pad + rl_pad - 1] == 0 { self.print_rl_crk_range(p_itr, p_high + 1);panic!(); }

                            self.crk_col.run_lengths[p_itr + rl_high] = self.crk_col.run_lengths[p_pad + rl_pad - 1];
                            // Fix P         to P + |rem| - 1 (inside padding)
                            self.crk_col.run_lengths[p_pad] = rem_size;
                            self.crk_col.run_lengths[p_pad + rem_size - 1] = rem_size;
                        }
                        // Move the rl marker for the main section of the high-side run to the end of the section.
                        self.crk_col.run_lengths[p_high - rl_itr + 1] = self.crk_col.run_lengths[p_high];
                        // Do the swaps
                        // Main: Swap I to I + rl[I] - 1 with H to H - rl[I] + 1
                        for i in 0..rl_itr {
                            self.crk_col.crk.swap(p_itr + i, p_high - rl_itr + 1 + i);
                            self.crk_col.base_idx.swap(p_itr + i, p_high - rl_itr + 1 + i);
                            self.crk_col.run_lengths.swap(p_itr + i, p_high - rl_itr + 1 + i);
                        }
                        // Padding: Swap I + rl[I] to I + rl[H] - 1 with H - rl[H] + 1 to H - rl[I]
                        for i in 0..pad_size {
                            self.crk_col.crk.swap(p_itr + rl_itr + i, p_high - rl_high + 1 + i);
                            self.crk_col.base_idx.swap(p_itr + rl_itr +  i, p_high - rl_high + 1 + i);
                            self.crk_col.run_lengths.swap(p_itr + rl_itr +  i, p_high - rl_high + 1 + i);
                        }
                    }
                    // Tighten H by rl[I]
                    // p_high -= rl_itr;
                } else {
                    // Do full, immediate swap
                    for i in 0..rl_itr {
                        self.crk_col.crk.swap(p_itr + i, p_high - rl_high + 1 + i);
                        self.crk_col.base_idx.swap(p_itr + i, p_high - rl_high + 1 + i);
                        self.crk_col.run_lengths.swap(p_itr + i, p_high - rl_high + 1 + i);
                    }

                    // Tighten H by rl[I]
                    p_high -= rl_itr;
                }

                // Tighten high
                while self.crk_col.crk[p_high] > x && p_high > p_low {
                    let mut rl = self.crk_col.run_lengths[p_high];
                    if self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                        while self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                            let inc = self.crk_col.run_lengths[p_high - rl];
                            if p_high < rl + inc {
                                break;
                            } else if p_high - (rl + inc) < p_low {
                                break;
                            }
                            rl += inc;
                        }
                        self.crk_col.run_lengths[p_high]            = rl;
                        self.crk_col.run_lengths[(p_high - rl) + 1] = rl;
                    }
                    p_high -= rl;
                }
            } else {
                // Advance itr
                let mut rl = self.crk_col.run_lengths[p_itr];
                while p_itr + rl < self.count {
                    if self.crk_col.crk[p_itr + rl] == x {
                        if p_itr + rl + self.crk_col.run_lengths[p_itr + rl] < self.count {
                            rl += self.crk_col.run_lengths[p_itr + rl];
                            self.crk_col.run_lengths[p_itr] = rl;
                            self.crk_col.run_lengths[p_itr + rl - 1] = rl;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                p_itr += self.crk_col.run_lengths[p_itr];
            }
        }

        if p_low > p_high {
            println!("x:{}, L:{}, I:{}, H:{}", x, p_low, p_itr, p_high);
            self.print_rl_crk();
            panic!();
        }

        // Memo
        self.crk_col.crk_idx.insert(x, p_low);
        self.crk_col.crk_idx.insert(x + 1, p_high + 1);
        self.get_indices(self.crk_col.base_idx[p_low..(p_high + 1)].iter())
    }

    // Counts the places where a given column equals a given value
    pub fn count_col_eq(&self, col: &str, eq: i64) -> i64 {
        self.get_col(col).v.iter().map(|&x|(x==eq)as i64).fold(0,|sum,x|sum+x) as i64
    }
}

// Returns an adjacency list built from the two vectors of adjacent nodes.
pub fn from_adjacency_vectors(src_node: Vec<i64>, dst_node: Vec<i64>, crk: &str) -> OverswapRLETable {
    let mut adjacency_list = OverswapRLETable::new();
    adjacency_list.new_columns(vec!["src", "dst"]);
    adjacency_list.insert(&mut map!{"src" => src_node, "dst" => dst_node});
    adjacency_list.set_crk_col(crk);
    adjacency_list
}