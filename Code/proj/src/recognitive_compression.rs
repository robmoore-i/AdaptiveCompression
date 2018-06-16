// Decomposed cracking
// Fragmentary, recognitive compression

// uses map! macro.

use cracker_index::AVLCrackerIndex;
use column::Column;
use column::IntCol;

use std::collections::HashMap;
use std::slice::Iter;

#[derive(Clone)]
pub struct ReCoTable {
    pub count: usize,
    pub crk_col_name: String,
    pub crk_col: IntCol,
    pub int_columns: HashMap<String, IntCol>,
}

impl ReCoTable {
    pub fn new() -> ReCoTable {
        ReCoTable {
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
                self.crk_col.v        = c.v.clone();
                self.crk_col.crk      = c.v.clone();
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

        let check_inserted_column_lengths = |v: &Vec<i64>, l: usize| if v.len() != l { panic!("insert: Int columns to be inserted do not have the same length") };

        for (k, v) in new_values.iter() {
            if self.int_columns.contains_key(&(k.to_string())) {
                match l_old {
                    Some(l) => check_inserted_column_lengths(v, l),
                    None    => l_old = Some(v.len()),
                };
            } else {
                match l_new {
                    Some(l) => check_inserted_column_lengths(v, l),
                    None    => l_new = Some(v.len()),
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

    pub fn get_values(&self, indices: Iter<usize>, col: &str) -> Vec<i64> {
        let mut buf = Vec::new();
        for &i in indices {
            buf.push(self.int_columns[&col.to_string()].v[i]);
        }
        buf
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

    pub fn get_indices(&self, indices: Iter<usize>) -> ReCoTable {
        let mut int_selection: HashMap<String, IntCol> = HashMap::new();
        for (name, col) in &self.int_columns {
            let mut v_buffer = Vec::with_capacity(indices.len());
            for &i in indices.clone() {
                v_buffer.push(col.v[i]);
            }
            let mut c_buffer = IntCol::empty();
            c_buffer.v = v_buffer;
            int_selection.insert(name.clone(), c_buffer);
        }

        let mut t = ReCoTable::new();
        t.int_columns = int_selection;
        t.count = indices.len();

        let mut indexed_crk_v = Vec::with_capacity(indices.len());
        if self.crk_col.crk.len() > 0 {
            let mut indexed_crk_col = Vec::with_capacity(indices.len());
            for &i in indices.clone() {
                indexed_crk_col.push(self.crk_col.crk[i]);
                indexed_crk_v.push(self.crk_col.v[i]);
            }
            t.crk_col.crk     = indexed_crk_col;
            t.crk_col.crk_idx = AVLCrackerIndex::new();
        } else {
            for &i in indices.clone() {
                indexed_crk_v.push(self.crk_col.v[i]);
            }
        }
        t.crk_col.v = indexed_crk_v;
        t
    }

    // Returns the elements of T where the cracker columns's value equals X
    pub fn cracker_select_specific(&mut self, x: i64, col: &str) -> Vec<i64> {
        self.cracker_select_in_three(x, x, true, true, col)
    }

    // Returns the elements of T where the cracker columns's value is between LOW and HIGH, with inclusivity given by INC_L and INC_H.
    pub fn cracker_select_in_three(&mut self, low: i64, high: i64, inc_l: bool, inc_h: bool, col: &str) -> Vec<i64> {
        let adjusted_low = low + !inc_l as i64;
        let adjusted_high = high - !inc_h as i64;
        // c_low(x)  <=> x outside catchment at low  end
        // c_high(x) <=> x outside catchment at high end
        let c_low  = |x| x < adjusted_low;
        let c_high = |x| x > adjusted_high;

        // Start with a pointer at both ends of the piece: p_low, p_high
        let mut p_low =  self.crk_col.crk_idx.lower_bound(&adjusted_low).unwrap_or(0);
        let mut p_high = self.crk_col.crk_idx.upper_bound(&(high + inc_h as i64)).unwrap_or(self.count);

        let is_uniform_column_piece = adjusted_low == adjusted_high && self.crk_col.crk_idx.contains(adjusted_low) && self.crk_col.crk_idx.contains(high + inc_h as i64);
        if is_uniform_column_piece {
            return self.get_values(self.crk_col.base_idx[p_low..p_high].iter(), col);
        }
        if p_high == self.count { p_high = self.count - 1 };
        if p_low  == self.count { p_low  = self.count - 1 };

        // while p_low is pointing at an element satisfying c_low,  move it forwards
        while c_low(self.crk_col.crk[p_low]) {
            p_low += 1;
            if p_low == self.count as usize {
                return vec![]
            }
        }

        // while p_high is pointing at an element satisfying c_high, move it backwards
        while c_high(self.crk_col.crk[p_high]) {
            if p_high == 0 {
                return vec![];
            }
            p_high -= 1;
        }

        if p_low == p_high {
            return self.get_values(self.crk_col.base_idx[p_low..(p_high + 1)].iter(), col);
        }

        let mut p_itr = p_low.clone();

        while p_itr <= p_high {
            if c_low(self.crk_col.crk[p_itr]) {
                self.crk_col.crk.swap(p_low, p_itr);
                self.crk_col.base_idx.swap(p_low, p_itr);
                while c_low(self.crk_col.crk[p_low]) {
                    p_low += 1;
                }
                if p_itr < p_low {
                    p_itr = p_low.clone();
                }
            } else if c_high(self.crk_col.crk[p_itr]) {
                self.crk_col.crk.swap(p_itr, p_high);
                self.crk_col.base_idx.swap(p_itr, p_high);
                while c_high(self.crk_col.crk[p_high]) {
                    p_high -= 1;
                }
            } else {
                p_itr += 1;
            }
        }

        self.crk_col.crk_idx.insert(low + !inc_l as i64, p_low);
        self.crk_col.crk_idx.insert(high + inc_h as i64, p_high + 1);
        self.get_values(self.crk_col.base_idx[p_low..p_itr].iter(), col)
    }

    // Counts the places where a given column equals a given value
    pub fn count_col_eq(&self, col: &str, eq: i64) -> i64 {
        self.get_i64_col(col).v.iter().map(|&x|(x==eq)as i64).fold(0,|sum,x|sum+x) as i64
    }
}

// Returns an adjacency list built from the two vectors of adjacent nodes.
pub fn from_adjacency_vectors(src_node: Vec<i64>, dst_node: Vec<i64>, crk: &str) -> ReCoTable {
    let mut adjacency_list = ReCoTable::new();
    adjacency_list.new_columns(vec!["src", "dst"]);
    adjacency_list.insert(&mut map!{"src" => src_node, "dst" => dst_node});
    adjacency_list.set_crk_col(crk);
    adjacency_list
}