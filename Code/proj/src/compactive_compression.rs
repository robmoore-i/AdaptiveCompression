// Decomposed cracking
// Fragmentary, compactive compression

// uses map! macro.

use cracker_index::AVLCrackerIndex;
use column::Column;
use column::IntCol;

use std::collections::HashMap;
use std::slice::Iter;
use std::ops::Range;

#[derive(Clone)]
pub struct CoCoTable {
    pub count: usize,
    pub crk_col_name: String,
    pub crk_col: IntCol,
    pub int_columns: HashMap<String, IntCol>,
}

impl CoCoTable {
    pub fn new() -> CoCoTable {
        CoCoTable {
            count: 0,
            crk_col_name: "".to_string(),
            crk_col: IntCol::empty(),
            int_columns: HashMap::new(),
        }
    }

    pub fn print_cols(&self) {
        println!("crk: {:?}", self.crk_col.crk);
        for (name, int) in self.int_columns.clone() {
            println!("{}: {:?}", name, int.v);
        }
    }

    pub fn set_crk_col(&mut self, col_name: &str) {
        self.crk_col_name = col_name.to_string();

        match self.int_columns.get_mut(&(col_name.to_string())) {
            Some(ref mut c) => {
                if c.crk.is_empty() {
                    c.crk = c.v.clone();
                }
                self.crk_col.v = c.crk.clone();
                self.crk_col.crk = c.crk.clone();
                self.crk_col.base_idx = (0..self.count).collect();
            },
            None => panic!("set_crk_col: no such col"),
        }
    }

    pub fn new_columns(&mut self, cols: Vec<&str>) {
        for name in cols {
            let column_was_overwritten = self.int_columns.insert(name.to_string(), IntCol::empty()).is_some();
            if column_was_overwritten {
                panic!("new_columns: Overwrote column: {}", name);
            }
        }
    }

    pub fn insert(&mut self, new_values: &mut HashMap<&str, Vec<i64>>) {
        // Check: self.columns.keys SUBSET-OF new_values.keys
        for k in self.int_columns.keys() {
            if !new_values.contains_key(&*(k.clone())) {
                panic!("insert: Must insert to all columns at a time")
            }
        }

        // Check: The length of the old or new- column entries must all be equal
        let mut l_old = None;
        let mut l_new = None;

        let check_inserted_column_lengths = |v: &Vec<i64>, l: usize| if v.len() != l { panic!("insert: (i64) Columns to be inserted do not have the same length") };

        for (k, v) in new_values.iter() {
            if self.int_columns.contains_key(&(k.to_string())) {
                match l_old {
                    Some(l) => check_inserted_column_lengths(v, l),
                    None => l_old = Some(v.len()),
                };
            } else {
                match l_new {
                    Some(l) => check_inserted_column_lengths(v, l),
                    None => l_new = Some(v.len()),
                };
            }
        }

        // Check: After insertion, the columns must all still have the same length
        match (l_new, l_old) {
            (Some(n), Some(o)) => {
                if n != o + self.count {
                    panic!("insert: (new & old-column) Values to be inserted are not the correct lengths")
                }
            },
            (Some(n), None) => {
                if n != self.count {
                    panic!("insert: (new-column-only) Values to be inserted are not the correct lengths")
                }
            },
            (None, _) => {},
        };

        // For every old-column entry, append the values to the current column
        // For every new-column entry, create the column and add the values
        for (k, v) in new_values.iter_mut() {
            if self.int_columns.contains_key(&(k.to_string())) {
                let c = self.int_columns.get_mut(&(k.to_string())).unwrap();
                c.append(v);
            } else {
                let mut new = IntCol::empty();
                new.append(v);
                self.int_columns.insert(k.to_string(), new);
            }
        }

        // Mark the increased size of the table
        if l_new.is_some() {
            self.count = l_new.unwrap();
        } else {
            self.count += l_old.unwrap();
        }
    }

    pub fn rearrange(&mut self, indices: Iter<usize>) {
        for col in self.int_columns.values_mut() {
            col.rearrange(indices.clone());
        }
        self.crk_col.rearrange(indices.clone());
    }

    pub fn get_i64_col(&self, col: &str) -> &IntCol {
        self.int_columns.get(&(col.to_string())).expect(&*("get_col: No column called ".to_string() + col))
    }

    pub fn get_indices(&self, base_indices: Iter<usize>, compressed_indices: Range<usize>) -> CoCoTable {
        let mut int_selection: HashMap<String, IntCol> = HashMap::new();
        for (name, col) in &self.int_columns {
            let mut v_buffer = Vec::with_capacity(base_indices.len());
            for &i in base_indices.clone() {
                v_buffer.push(col.v[i]);
            }
            let mut c_buffer = IntCol::empty();
            c_buffer.v = v_buffer;
            int_selection.insert(name.clone(), c_buffer);
        }

        let mut t = CoCoTable::new();
        t.int_columns = int_selection;
        t.count = base_indices.len();

        let mut indexed_crk_v = Vec::with_capacity(base_indices.len());
        if self.crk_col.crk.len() > 0 {
            let mut indexed_crk_col = Vec::with_capacity(base_indices.len());
            for i in compressed_indices.clone() {
                indexed_crk_col.push(self.crk_col.crk[i]);
            }
            for &i in base_indices.clone() {
                indexed_crk_v.push(self.crk_col.v[i]);
            }
            t.crk_col.crk = indexed_crk_col;
            t.crk_col.crk_idx = AVLCrackerIndex::new();
        } else {
            for &i in base_indices.clone() {
                indexed_crk_v.push(self.crk_col.v[i]);
            }
        }
        t.crk_col.v = indexed_crk_v;
        t
    }

    // Compact the cracker column if there is an opportunity to do so, given the recent crk_idx addition of V->I
    pub fn compact(&mut self, v: i64, i: usize) {
        if self.crk_col.ofs.is_empty() {
            self.crk_col.ofs = (0..self.count).collect();
        }
        if let Some(j) = self.crk_col.crk_idx.get(v + 1) {
            if i >= j - 1 { return; }
            // Compress v
            self.crk_col.crk.drain((i + 1)..j);
            self.crk_col.ofs.drain((i + 1)..j);
            self.crk_col.crk_idx.subtract_where_greater_than(v, j - i - 1);
        }
        if let Some(j) = self.crk_col.crk_idx.get(v - 1) {
            if j >= i - 1 { return; }
            // Compress v - 1
            self.crk_col.crk.drain((j + 1)..i);
            self.crk_col.ofs.drain((j + 1)..i);
            self.crk_col.crk_idx.subtract_where_greater_than(v - 1, i - j - 1);
        }
    }

    pub fn decompress_index(&self, compressed_index: usize) -> CoCoTable {
        let offset = self.crk_col.ofs[compressed_index];

        let next_offset = if compressed_index >= self.crk_col.ofs.len() - 1 {
            self.crk_col.base_idx.len()
        } else {
            self.crk_col.ofs[compressed_index + 1]
        };

        let index_range = offset..next_offset;
        let mut base_indices = Vec::with_capacity(next_offset - offset);
        for i in index_range {
            base_indices.push(self.crk_col.base_idx[i]);
        }

        let mut int_selection: HashMap<String, IntCol> = HashMap::new();
        for (name, col) in &self.int_columns {
            let mut v_buffer = Vec::with_capacity(base_indices.len());
            for i in base_indices.clone() {
                v_buffer.push(col.v[i]);
            }
            let mut c_buffer = IntCol::empty();
            c_buffer.v = v_buffer;
            int_selection.insert(name.clone(), c_buffer);
        }
        let mut t = CoCoTable::new();
        t.int_columns = int_selection;
        t.count = base_indices.len();
        t
    }

    // Returns the elements of T where the cracker columns's value equals X
    pub fn cracker_select_specific(&mut self, x: i64) -> CoCoTable {
        self.cracker_select_in_three(x, x, true, true)
    }

    // Returns the elements of T where the cracker columns's value is between LOW and HIGH, with inclusivity given by INC_L and INC_H.
    pub fn cracker_select_in_three(&mut self, low: i64, high: i64, inc_l: bool, inc_h: bool) -> CoCoTable {

        // PHASE 0: Setup

        // If column hasn't been cracked before, copy it, and copy a reference to the current
        // indices of the base table.
        if self.crk_col.crk.is_empty() {
            self.crk_col.crk = self.crk_col.v.clone();
            self.crk_col.base_idx = (0..self.count).collect();
        }

        if self.crk_col.ofs.is_empty() {
            self.crk_col.ofs = (0..self.count).collect();
        }

        let adjusted_low = low + !inc_l as i64;
        let adjusted_high = high - !inc_h as i64;
        // c_low(x)  <=> x outside catchment at low  end
        // c_high(x) <=> x outside catchment at high end
        let c_low = |x| x < adjusted_low;
        let c_high = |x| x > adjusted_high;

        let count = self.crk_col.crk.len();

        // Start with a pointer at both ends of the array: p_low, p_high

        let mut p_low = self.crk_col.crk_idx.lower_bound(&adjusted_low).unwrap_or(0);
        let mut p_high = self.crk_col.crk_idx.upper_bound(&(high + inc_h as i64)).unwrap_or((count - 1) as usize);
        if p_high >= count { p_high = count - 1; }
        if p_low >= count { p_low = count - 1; }

        // PHASE 1: Move pointers inwards

        // while p_low is pointing at an element satisfying c_low,  move it forwards
        while c_low(self.crk_col.crk[p_low]) {
            p_low += 1;
            if p_low == count as usize {
                return self.get_indices(vec![].iter(), 0..0);
            }
        }

        // while p_high is pointing at an element satisfying c_high, move it backwards
        while c_high(self.crk_col.crk[p_high]) {
            p_high -= 1;
            if p_high == 0 && c_high(self.crk_col.crk[p_high]) {
                return self.get_indices(vec![].iter(), 0..0);
            }
        }

        // If the vertex is compressed/contains a single entry, return that.
        if p_low == p_high {
            let v = self.crk_col.crk[p_low];
            let w = self.crk_col.crk[p_low] + 1;
            if self.crk_col.crk_idx.contains(v) && self.crk_col.crk_idx.contains(w) {
                return self.decompress_index(p_low);
            } else {
                return self.get_indices(vec![self.crk_col.base_idx[self.crk_col.ofs[p_low]]].iter(), p_low..(p_high + 1));
            }
        }

        // PHASE 2: Main loop

        let mut p_itr = p_low.clone();

        while p_itr <= p_high {
            if c_low(self.crk_col.crk[p_itr]) {
                self.crk_col.crk.swap(p_low, p_itr);
                self.crk_col.base_idx.swap(self.crk_col.ofs[p_low], self.crk_col.ofs[p_itr]);
                while c_low(self.crk_col.crk[p_low]) {
                    p_low += 1;
                }
                if p_itr < p_low {
                    p_itr = p_low.clone();
                }
            } else if c_high(self.crk_col.crk[p_itr]) {
                self.crk_col.crk.swap(p_itr, p_high);
                self.crk_col.base_idx.swap(self.crk_col.ofs[p_itr], self.crk_col.ofs[p_high]);
                while c_high(self.crk_col.crk[p_high]) {
                    p_high -= 1;
                }
            } else {
                p_itr += 1;
            }
        }

        // PHASE 3: Compression

        let high_v = high + inc_h as i64;
        if !(high_v > self.crk_col.crk.len() as i64) {
            self.crk_col.crk_idx.insert(high_v, p_itr);
            self.compact(high_v, p_itr);
        }

        let low_v = adjusted_low;
        self.crk_col.crk_idx.insert(low_v, p_low);
        self.compact(low_v, p_low);

        // PHASE 4: Decompression

        if adjusted_low == adjusted_high && p_low <= p_high {
            let index_of_x = self.crk_col.crk_idx.get(adjusted_low).unwrap();
            self.decompress_index(index_of_x)
        } else {
            let compressed_high_ptr = if p_itr >= count { p_itr } else { self.crk_col.ofs[p_itr] };
            self.get_indices(self.crk_col.base_idx[self.crk_col.ofs[p_low]..compressed_high_ptr].iter(), p_low..p_itr)
        }
    }

    // Counts the places where a given column equals a given value
    pub fn count_col_eq(&self, col: &str, eq: i64) -> i64 {
        self.get_i64_col(col).v.iter().map(|&x|(x==eq)as i64).fold(0,|sum,x|sum+x) as i64
    }
}

// Returns an adjacency list built from the two vectors of adjacent nodes.
pub fn from_adjacency_vectors(src_node: Vec<i64>, dst_node: Vec<i64>, crk: &str) -> CoCoTable {
    let mut adjacency_list = CoCoTable::new();
    adjacency_list.new_columns(vec!["src", "dst"]);
    adjacency_list.insert(&mut map!{"src" => src_node, "dst" => dst_node});
    adjacency_list.set_crk_col(crk);
    adjacency_list
}
