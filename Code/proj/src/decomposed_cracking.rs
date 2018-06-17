// Decomposed cracking
// No compression

// uses map! macro.

use cracker_index::AVLCrackerIndex;
use column::Column;
use column::IntCol;

use std::collections::HashMap;
use std::slice::Iter;

#[derive(Clone)]
pub struct DeCrackedTable {
    pub count: usize,
    pub crk_col: IntCol,
    pub columns: HashMap<String, IntCol>,
}

impl DeCrackedTable {
    pub fn new() -> DeCrackedTable {
        DeCrackedTable {
            count: 0,
            crk_col: IntCol::empty(),
            columns: HashMap::new()
        }
    }

    pub fn new_columns(&mut self, col_names: Vec<String>) {
        for col in col_names {
            self.columns.insert(col, IntCol::empty());
        }
    }

    pub fn set_crk_col(&mut self, col_name: String) {
        match self.columns.get(&col_name) {
            Some(ref c) => {
                self.crk_col.v        = c.v.clone();
                self.crk_col.crk      = c.v.clone();
                self.crk_col.base_idx = (0..self.count).collect();
            },
            None => panic!("set_crk_col: no such col"),
        };
    }

    // TODO: Improve exception handling in this function
    pub fn insert(&mut self, new_values: &mut HashMap<String, Vec<i64>>) {
        let mut n_new_tuples = 0;
        for (key, val) in self.columns.iter_mut() {
            let new_elements = new_values.get_mut(key).unwrap();
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

    pub fn get_col(&self, col: String) -> Option<&IntCol> {
        self.columns.get(&col)
    }

    pub fn get_indices(&self, indices: Iter<usize>) -> DeCrackedTable {
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

        let mut t = DeCrackedTable::new();
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

    pub fn get_values(&self, indices: Iter<usize>, col: &str) -> Vec<i64> {
        let mut buf = Vec::new();
        for &i in indices {
            buf.push(self.columns[&col.to_string()].v[i]);
        }
        buf
    }

    pub fn rearrange(&mut self, indices: Iter<usize>) {
        for col in self.columns.values_mut() {
            col.rearrange(indices.clone());
        }
        self.crk_col.rearrange(indices.clone());
    }

    // Returns the elements of T where the cracker columns's value equals X
    pub fn cracker_select_specific(&mut self, x: i64, col: &str) -> Vec<i64> {
        // Start with a pointer at both ends of the array: p_low, p_high
        let mut p_low = self.crk_col.crk_idx.lower_bound(&x).unwrap_or(0);
        let mut p_high = self.crk_col.crk_idx.upper_bound(&(x + 1)).unwrap_or(self.count) - 1;
        if p_high + 1 == 0 { return vec![] }; // Value lower than lowest value in column - No results.

        // while p_low is pointing at an element satisfying c_low,  move it forwards
        while self.crk_col.crk[p_low] < x {
            p_low += 1;
            if p_low == self.count as usize {
                return vec![];
            }
        }

        // while p_high is pointing at an element satisfying c_high, move it backwards
        while self.crk_col.crk[p_high] > x {
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
            if self.crk_col.crk[p_itr] < x {
                self.crk_col.crk.swap(p_low, p_itr);
                self.crk_col.base_idx.swap(p_low, p_itr);
                while self.crk_col.crk[p_low] < x {
                    p_low += 1;
                }
                if p_itr < p_low {
                    p_itr = p_low.clone();
                }
            } else if self.crk_col.crk[p_itr] > x {
                self.crk_col.crk.swap(p_itr, p_high);
                self.crk_col.base_idx.swap(p_itr, p_high);
                while self.crk_col.crk[p_high] > x {
                    p_high -= 1;
                }
            } else {
                p_itr += 1;
            }
        }

        self.crk_col.crk_idx.insert(x, p_low);
        self.crk_col.crk_idx.insert(x + 1, p_high + 1);
        self.get_values(self.crk_col.base_idx[p_low..(p_high + 1)].iter(), col)
    }

    // Counts the places where a given column equals a given value
    pub fn count_col_eq(&self, col: &str, eq: i64) -> i64 {
        self.get_col(col.to_string()).unwrap().v.iter().map(|&x|(x==eq)as i64).fold(0,|sum,x|sum+x) as i64
    }
}

// Returns an adjacency list built from the two vectors of adjacent nodes.
pub fn from_adjacency_vectors(src_node: Vec<i64>, dst_node: Vec<i64>, crk: &str) -> DeCrackedTable {
    let mut adjacency_list = DeCrackedTable::new();
    adjacency_list.new_columns(vec!["src".to_string(), "dst".to_string()]);
    adjacency_list.insert(&mut map!{"src".to_string() => src_node, "dst".to_string() => dst_node});
    adjacency_list.set_crk_col(crk.to_string());
    adjacency_list
}