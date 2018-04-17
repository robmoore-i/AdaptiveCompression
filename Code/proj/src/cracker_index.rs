// Implements the functions required for the cracker index.
// Maps i64 -> usize
#[derive(Clone)]
pub struct ArrayCrackerIndex {
    pub index: Vec<Option<usize>>
}
impl ArrayCrackerIndex {
    pub fn new() -> ArrayCrackerIndex {
        ArrayCrackerIndex { index: Vec::new() }
    }

    pub fn insert(&mut self, key: i64, data: usize) {
        if key < 0 {
            return;
        }
        while self.index.len() <= key as usize {
            self.index.push(None);
        }
        self.index[key as usize] = Some(data);
    }

    pub fn delete(&mut self, key: i64) {
        if self.index.len() > key as usize {
            self.index.remove(key as usize);
        }
    }

    pub fn get(&self, key: i64) -> Option<usize> {
        if self.index.len() <= key as usize {
            None
        } else {
            self.index[key as usize]
        }
    }

    pub fn get_or(self, key: i64, default: usize) -> usize {
        self.get(key).map_or(default, |data| data)
    }

    pub fn contains(&self, key: i64) -> bool {
        self.get(key).is_some()
    }

    pub fn empty(&self) -> bool { self.index.len() == 0 }

    // Returns the smallest key >= key
    pub fn upper_bound(&self, key: &i64) -> Option<usize> {
        if *key < 0 {
            return None;
        }
        let mut k = (*key) as usize;
        if k >= self.index.len() {
            None
        } else {
            while self.index[k].is_none() && k < self.index.len() - 1 {
                k += 1;
            }
            self.index[k]
        }
    }

    // Returns the largest key <= key
    pub fn lower_bound(&self, key: &i64) -> Option<usize> {
        if *key < 0 {
            return None;
        }
        let mut k = (*key) as usize;
        if k >= self.index.len() {
            None
        } else {
            while self.index[k].is_none() && k > 0 {
                k -= 1;
            }
            self.index[k]
        }
    }

    // For all keys > THRESHOLD, subtract their value by AMOUNT.
    // Assumed that threshold is a key in the current index
    pub fn subtract_where_greater_than(&mut self, threshold: i64, amount: usize) {
        let mut k = 1 + threshold as usize;
        let l = self.index.len();
        while k < l {
            if let Some(d) = self.index[k].as_mut() {
                *d -= amount;
            }
            k += 1;
        }
    }
}