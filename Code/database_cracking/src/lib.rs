// All credit for this beautiful AVL tree implementation belongs to: https://github.com/eqv/avl_tree
// What a legend
// I added functions for max_before to go with min_after, and also added functions to the Tree struct
// to go with them. The min_after layout changed a bit, but the underlying algorithm is the same.
pub mod avl {
    use std::cmp;
    use std::cmp::Ordering;
    use std::fmt::Debug;

    #[derive(Clone)]
    pub struct Node<K: Ord, D:Debug> {
        key: K,
        data: D,
        height: u64,
        left:  Option<Box<Node<K,D>>>,
        right: Option<Box<Node<K,D>>>,
    }

    impl<K:Ord, D:Debug> Node<K,D> {
        pub fn new(key: K, data: D) -> Node<K,D> {
            Node::<K,D>{key: key, data: data, height: 1, left: None, right: None}
        }
    }
    
    fn height<K:Ord,D:Debug>(node: &Option<Box<Node<K,D>>>) -> u64  {
        return node.as_ref().map_or(0, |succ| succ.height)
    }
    
    // Perform a single right rotation on this (sub) tree
    fn rotate_right<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let mut new_root_box = root.left.take().expect("AVL broken");
        root.left = new_root_box.right.take();
        update_height(&mut root);
        new_root_box.right = Some(root);
        update_height(&mut new_root_box);
        return new_root_box
    }

    // Perform a single left rotation on this (sub) tree
    fn rotate_left<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let mut new_root_box = root.right.take().expect("AVL broken");
        root.right = new_root_box.left.take();
        update_height(&mut root);
        new_root_box.left = Some(root);
        update_height(&mut new_root_box);
        return new_root_box
    }

    // Performs a rotation that counteracts the fact that the left successor is too high
    fn rotate_left_successor<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let left = root.left.take().expect("AVL broken");
        if height(&left.left) < height(&left.right) {
            let rotated = rotate_left(left);
            root.left = Some(rotated);
            update_height(&mut root);
        }
        else{
            root.left = Some(left);
        }
        rotate_right(root)
    }

    // Performs a rotation that counteracts the fact that the right successor is too high
    fn rotate_right_successor<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let right = root.right.take().expect("AVL broken");
        if height(&right.left) > height(&right.right) {
            let rotated = rotate_right(right);
            root.right = Some(rotated);
            update_height(&mut root);
        }
        else {
            root.right = Some(right)
        }
        rotate_left(root)
    }

    fn diff_of_successors_height<K:Ord,D:Debug>(root: &Box<Node<K,D>>) -> i32 {
        let l = height(&root.left);
        let r = height(&root.right);
        (l as i32) - (r as i32)
    }


    // Apply all necessary rotations on root.
    fn rotate_if_necessary<K:Ord,D:Debug>(root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let diff  = diff_of_successors_height(&root);
        if -1 <= diff && diff <= 1 {return root}
        match diff {
            2  => rotate_left_successor::<K,D>(root),
            -2 => rotate_right_successor::<K,D>(root),
            _  => unreachable!()
        }
    }

    // Update the cached height of root. To call this function make sure that the cached values of
    // both children of root ar up to date.
    fn update_height<K:Ord,D:Debug>(root: &mut Node<K,D>) {
        root.height = cmp::max( height(&root.left), height(&root.right) )+1;
    }

    // Recursively insert the (key,data) pair into the given optional succesor and return its new value
    fn insert_in_successor<K:Ord,D:Debug>(key: K, data: D, successor: Option<Box<Node<K,D>>>) -> Option<Box<Node<K,D>>> {
                Some(match successor {
                    Some(succ) => insert(key, data, succ),
                    None       => Box::new(Node::new(key, data))
                })
    }

    // Inserts the given data under the key in the tree root. It will replace old data stored
    // under this key if it was allready used in the tree. The resulting tree will be returned
    // (its root may now differ due to rotations, thus the old root is moved into the function)
    pub fn insert<K:Ord,D:Debug>(key: K, data: D, mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        match root.key.cmp(&key) {
            Ordering::Equal   => { root.data  = data; return root },
            Ordering::Less    => root.right = insert_in_successor(key, data, root.right.take()),
            Ordering::Greater => root.left  = insert_in_successor(key,data, root.left.take())
        }
        update_height(&mut *root);
        return rotate_if_necessary(root)
    }

    // Returns a read only reference to the data stored under key in the tree given by root
    pub fn search<'a, K:Ord,D:Debug>(key: &K, root: &'a Box<Node<K,D>>) -> Option<&'a D> {
        search_pair(key,root).map(|(_,v)| v )
    }

    // Returns a read only reference pair to the data stored under key in the tree given by root
    pub fn search_pair<'a, K:Ord,D:Debug>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
        match root.key.cmp(key) {
            Ordering::Equal   => Some((&root.key, &root.data)),
            Ordering::Less    => root.right.as_ref().map_or(None, |succ| search_pair(key, succ)),
            Ordering::Greater => root.left.as_ref().map_or(None, |succ| search_pair(key, succ))
        }
    }

    // Returns the smallest key value pair (k, v) s.t. k >= given key.
    pub fn min_after<'a, K:Ord,D:Debug>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
        match root.key.cmp(key) {
            Ordering::Equal   => Some((&root.key, &root.data)),           
            Ordering::Less    => {
                match root.right {
                    Some(ref succ) => min_after(key, &succ),
                    None           => None
                }
            },
            Ordering::Greater => {
                match root.left {
                    Some(ref succ) => min_after(key, &succ).or(Some((&root.key,&root.data))),
                    None           => Some((&root.key, &root.data))
                }
            }
        }
    }
    
    // Returns the greatest key value pair (k, v) s.t. k  <= given key.
    pub fn max_before<'a, K:Ord,D:Debug>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
        match root.key.cmp(key) {
            Ordering::Equal   => Some((&root.key, &root.data)),
            Ordering::Less    => {
                match root.right {
                    Some(ref succ) => max_before(key, &succ).or(Some((&root.key,&root.data))),
                    None           => Some((&root.key, &root.data))
                }
            }
            Ordering::Greater => {
                match root.left {
                    Some(ref succ) => max_before(key, &succ),
                    None           => None
                }
            }
        }
    }
    
    // Returns the minimal key,value pair within this tree
    pub fn min_pair<K:Ord,D:Debug>(root: &Box<Node<K,D>>) -> (&K,&D) {
        root.left.as_ref().map_or((&root.key,&root.data), min_pair)
    }

    // Returns the maximal key,value pair within this tree
    pub fn max_pair<K:Ord,D:Debug>(root: &Box<Node<K,D>>) -> (&K,&D) {
        root.right.as_ref().map_or((&root.key,&root.data), max_pair)
    }

    // Returns the minimal value within this tree
    pub fn min<K:Ord,D:Debug>(root: &Box<Node<K,D>>) -> &D {
        root.left.as_ref().map_or(&root.data, min)
    }

    // Returns the minimal value within this tree
    pub fn max<K:Ord,D:Debug>(root: &Box<Node<K,D>>) -> &D {
        root.right.as_ref().map_or(&root.data, max)
    }

    // Will update_heights and rotate the node if necessary, returns the rotated node
    fn updated_node<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        update_height(&mut root);
        rotate_if_necessary(root)
    }

    // Performs recursive `drop_and_get_min` if a left  since a successor is available
    fn drop_min_from_left<K:Ord,D:Debug>(mut root : Box<Node<K,D>>, left: Box<Node<K,D>>) -> (Option<Box<Node<K,D>>>,Box<Node<K,D>>) {
        let (new_left, min) = drop_min(left);
        root.left = new_left;
        (Some(updated_node(root)),min)
    }

    // Finds the minimal value below root and returns a new (optional) tree where the minimal value has been
    // removed and the (optional) minimal node as tuple (new_tree, min);
    fn drop_min<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> (Option<Box<Node<K,D>>>, Box<Node<K,D>>) {
        match root.left.take() {
            Some(left) => drop_min_from_left(root, left),
            None => (root.right.take(), root)
        }
    }

    // Return a new AVL tree, as the combination of two subtrees with max(l) <= min(r)
    fn combine_two_subtrees<K:Ord,D:Debug>(l: Box<Node<K,D>>, r: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let (remaining_tree, min) = drop_min(r);
        let mut new_root = min;
        new_root.left = Some(l);
        new_root.right = remaining_tree;
        updated_node(new_root)
    }

    // Return a new AVL tree, where the root has been removed
    fn delete_root<K:Ord,D:Debug>(mut root: Box<Node<K,D>>) -> Option<Box<Node<K,D>>> {
        match ( root.left.take(), root.right.take() ) {
            ( None,     None)    => None,
            ( Some(l),  None)    => Some(l),
            ( None,     Some(r)) => Some(r),
            ( Some(l),  Some(r)) => Some(combine_two_subtrees(l,r))
        }
    }

    // Deletes `key` from the tree `root`. Returns either `Some` tree or if the resilting tree is
    // empty: None.
    pub fn delete<K:Ord,D:Debug>(key: K, mut root: Box<Node<K,D>>) -> Option<Box<Node<K,D>>> {
        match root.key.cmp(&key) {
            Ordering::Equal =>  return delete_root(root),
            Ordering::Less  => {
                if let Some(succ) = root.right.take() {
                    root.right = delete(key, succ);
                    return Some(updated_node(root))
                }
            },
            Ordering::Greater => {
                if let Some(succ) = root.left.take() {
                    root.left = delete(key, succ);
                    return Some(updated_node(root))
                }
            }
        }
        return Some(root);
    }

    #[derive(Clone)]
    pub struct AVLTree<K:Ord+Copy+Debug,D:Debug> {
        pub root: Option<Box<Node<K,D>>>
    }

    impl <K:Ord+Copy+Debug,D:Debug> AVLTree<K,D> {
        pub fn new() -> AVLTree<K,D> {
            AVLTree{root: None}
        }

        pub fn insert(&mut self, key: K, data: D) {
            match self.root.take() {
                Some(box_to_node) => self.root = Some(insert::<K,D>(key, data, box_to_node)),
                None              => self.root = Some(Box::new(Node::new(key,data))),
            }
        }

        pub fn delete(&mut self, key: K) {
            match self.root.take() {
                Some(box_to_node) => self.root = delete(key,box_to_node),
                None              => return
            }
        }

        pub fn get(&self, key: K) -> Option<&D> {
            match self.root {
                Some(ref box_to_node) => search(&key, box_to_node),
                None                  => None
            }
        }

        pub fn get_or<'a>(&'a self, key: K, default: &'a D) -> &D {
            self.get(key).map_or(default, |data| data)
        }

        pub fn contains(&self, key: K) -> bool {
            self.get(key).is_some()
        }

        pub fn empty(&self) -> bool { self.root.is_none() }

        // Returns the smallest key >= key
        pub fn upper_bound(&self, key: &K) -> Option<&D> {
            match self.root {
                Some(ref tree) => {
                    match min_after(key, tree) {
                         Some((_k, v)) => Some(v),
                         None          => None
                    }
                },
                None => None
            }            
        }

        // Returns the largest key <= key
        pub fn lower_bound(&self, key: &K) -> Option<&D> {
            match self.root {
                Some(ref tree) => {
                    match max_before(key, tree) {
                         Some((_k, v)) => Some(v),
                         None          => None
                    }
                },
                None => None
            }            
        }
    }
}

// Implements the functions required for the cracker index.
// Maps i64 -> usize
pub mod cracker_index {
    #[derive(Clone)]
    pub struct CrackerIndex {
        pub index: Vec<Option<usize>>
    }

    impl CrackerIndex {
        pub fn new() -> CrackerIndex {
            CrackerIndex { index: Vec::new() }
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
    }
}

// Implementation of Database
pub mod db {
    extern crate time;

    use avl::*;
    use cracker_index::*;
    use std::collections::HashMap;
    use std::slice::Iter;
    use std::fmt::Debug;
    use self::time::PreciseTime;
    use self::time::SteadyTime;

    pub trait Column {
        type Item;
        fn empty() -> Self;
        fn rearrange(&mut self, indices: Iter<usize>);
        fn at(self, idx: usize) -> Self::Item;
        fn append(&mut self, values: &mut Vec<Self::Item>);
    }

    #[derive(Clone)]
    pub struct FloatCol {
        pub v: Vec<f64>,
        pub base_idx: Vec<usize>,
    }

    impl Column for FloatCol {
        type Item = f64;

        fn empty() -> FloatCol {
            FloatCol {
                v: Vec::new(),
                base_idx: Vec::new()
            }
        }

        fn rearrange(&mut self, indices: Iter<usize>) {
            let mut replacement_v = Vec::with_capacity(self.v.len());
            for &i in indices.clone() {
                replacement_v.push(self.v[i]);
            }
            self.v = replacement_v;
            self.base_idx = Vec::new();
        }

        fn at(self, idx: usize) -> f64 {
            self.v[idx]
        }

        fn append(&mut self, values: &mut Vec<f64>) {
            self.v.append(values);
        }
    }

    #[derive(Clone)]
    pub struct CrackableCol {
        // Original
        pub v: Vec<i64>,

        // Cracked
        pub crk: Vec<i64>,

        // Cracker index - for a value v, stores the index p such that
        // for all i < p: c[i] < v. That is - Every value before p in the column
        // is less than v.
        pub crk_idx: CrackerIndex,

        // Base index - maintains an index into the base columns of the table for alignment
        // during tuple reconstruction.
        pub base_idx: Vec<usize>,
    }

    impl Column for CrackableCol {
        type Item = i64;

        fn empty() -> CrackableCol {
            CrackableCol {
                v: Vec::new(),
                crk:Vec::new(),
                crk_idx: CrackerIndex::new(),
                base_idx: Vec::new()
            }
        }

        fn rearrange(&mut self, indices: Iter<usize>) {
            let mut replacement_v = Vec::with_capacity(self.v.len());
            for &i in indices.clone() {
                replacement_v.push(self.v[i]);
            }
            self.v = replacement_v;

            // Could be optimised for nested queries
            self.crk = Vec::new();
            self.crk_idx = CrackerIndex::new();
            self.base_idx = Vec::new();
        }

        fn at(self, idx: usize) -> i64 {
            self.v[idx]
        }

        fn append(&mut self, values: &mut Vec<i64>) {
            self.v.append(values);
        }
    }

    macro_rules! t {
        ($work:block, $tvar:ident) => {
            let start = PreciseTime::now();
            $work;
            let end = PreciseTime::now();
            $tvar = $tvar + start.to(end);
        };
    }

    #[derive(Clone)]
    pub struct Table {
        pub count: usize,
        pub crk_col_name: String,
        pub crk_col: CrackableCol,
        pub i64_columns: HashMap<String, CrackableCol>,
        pub f64_columns: HashMap<String, FloatCol>,
    }

    impl Table {
        pub fn new() -> Table {
            Table {
                count: 0,
                crk_col_name: "".to_string(),
                crk_col: CrackableCol::empty(),
                i64_columns: HashMap::new(),
                f64_columns: HashMap::new()
            }
        }

        pub fn print_cols(&self) {
            println!("crk: {:?}", self.crk_col.crk);
            for (name, int) in self.i64_columns.clone() {
                println!("{}: {:?}", name, int.v);
            }
            for (name, float) in self.f64_columns.clone() {
                println!("{}: {:?}", name, float.v);
            }
        }

        pub fn set_crk_col(&mut self, col_name: &str) {
            self.crk_col_name = col_name.to_string();

            match self.i64_columns.get_mut(&(col_name.to_string())) {
                Some(ref mut c) => {
                    if c.crk.is_empty() {
                        c.crk = c.v.clone();
                    }
                    self.crk_col.v        = c.crk.clone();
                    self.crk_col.crk      = c.crk.clone();
                    self.crk_col.base_idx = (0..self.count).collect();
                },
                None => panic!("set_crk_col: no such col"),
            }
        }

        pub fn new_columns(&mut self, cols: HashMap<&str, char>) {
            let types = "jf".to_string();
            for (name, t) in cols {
                if !types.contains(t) {
                    panic!("new_columns: Can't create a column with type: {}", t);
                }
                let was_column_overwritten = match t {
                    'j' => self.i64_columns.insert(name.to_string(), CrackableCol::empty()).is_some(),
                    'f' => self.f64_columns.insert(name.to_string(), FloatCol::empty()).is_some(),
                    _   => panic!("new_columns: failed to catch an invalid type: {}", t),
                };
                if was_column_overwritten {
                    panic!("new_columns: Overwrote column: {} with type: {}", name, t);
                }
            }
        }

        pub fn insert_multityped(&mut self, new_ints: &mut HashMap<&str, Vec<i64>>, new_floats: &mut HashMap<&str, Vec<f64>>) {
            // Check: self.columns.keys SUBSET-OF new_values.keys
            for k in self.i64_columns.keys() {
                if !new_ints.contains_key(&*(k.clone())) {
                    panic!("insert: (i64) Tried to add tuples with nulls")
                }
            }
            for k in self.f64_columns.keys() {
                if !new_floats.contains_key(&*(k.clone())) {
                    panic!("insert: (f64) Tried to add tuples with nulls")
                }
            }

            // Check: The length of the old or new- column entries must all be equal
            let mut l_old = None;
            let mut l_new = None;

            #[inline] let check_length_ints   = |v: &Vec<i64>, l: usize| if v.len() != l { panic!("insert: (i64) Columns to be inserted do not have the same length") };
            #[inline] let check_length_floats = |v: &Vec<f64>, l: usize| if v.len() != l { panic!("insert: (f64) Columns to be inserted do not have the same length") };

            for (k, v) in new_ints.iter() {
                if self.i64_columns.contains_key(&(k.to_string())) {
                    match l_old {
                        Some(l) => check_length_ints(v, l),
                        None    => l_old = Some(v.len()),
                    };
                } else {
                    match l_new {
                        Some(l) => check_length_ints(v, l),
                        None    => l_new = Some(v.len()),
                    };
                }
            }
            for (k, v) in new_floats.iter() {
                if self.f64_columns.contains_key(&(k.to_string())) {
                    match l_old {
                        Some(l) => check_length_floats(v, l),
                        None    => panic!("insert: (f64) Unreachable"),
                    };
                } else {
                    match l_new {
                        Some(l) => check_length_floats(v, l),
                        None    => panic!("insert: (f64) Unreachable"),
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
            for (k, v) in new_ints.iter_mut() {
                if self.i64_columns.contains_key(&(k.to_string())) {
                    let c = self.i64_columns.get_mut(&(k.to_string())).unwrap();
                    c.append(v);
                } else {
                    let mut new = CrackableCol::empty();
                    new.append(v);
                    self.i64_columns.insert(k.to_string(), new);
                }
            }
            for (k, v) in new_floats.iter_mut() {
                if self.f64_columns.contains_key(&(k.to_string())) {
                    let c = self.f64_columns.get_mut(&(k.to_string())).unwrap();
                    c.append(v);
                } else {
                    let mut new = FloatCol::empty();
                    new.append(v);
                    self.f64_columns.insert(k.to_string(), new);
                }
            }

            // Mark the increased size of the table
            if l_new.is_some() {
                self.count = l_new.unwrap();
            } else {
                self.count += l_old.unwrap();
            }
        }

        pub fn insert(&mut self, new_ints: &mut HashMap<&str, Vec<i64>>) {
            // Check: self.columns.keys SUBSET-OF new_values.keys
            for k in self.i64_columns.keys() {
                if !new_ints.contains_key(&*(k.clone())) {
                    panic!("insert: (i64) Tried to add tuples with nulls")
                }
            }

            // Check: The length of the old or new- column entries must all be equal
            let mut l_old = None;
            let mut l_new = None;

            #[inline] let check_length_ints   = |v: &Vec<i64>, l: usize| if v.len() != l { panic!("insert: (i64) Columns to be inserted do not have the same length") };

            for (k, v) in new_ints.iter() {
                if self.i64_columns.contains_key(&(k.to_string())) {
                    match l_old {
                        Some(l) => check_length_ints(v, l),
                        None    => l_old = Some(v.len()),
                    };
                } else {
                    match l_new {
                        Some(l) => check_length_ints(v, l),
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
            for (k, v) in new_ints.iter_mut() {
                if self.i64_columns.contains_key(&(k.to_string())) {
                    let c = self.i64_columns.get_mut(&(k.to_string())).unwrap();
                    c.append(v);
                } else {
                    let mut new = CrackableCol::empty();
                    new.append(v);
                    self.i64_columns.insert(k.to_string(), new);
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
            for col in self.i64_columns.values_mut() {
                col.rearrange(indices.clone());
            }
            for col in self.f64_columns.values_mut() {
                col.rearrange(indices.clone());
            }
            self.crk_col.rearrange(indices.clone());
        }

        pub fn get_i64_col(&self, col: &str) -> &CrackableCol {
            self.i64_columns.get(&(col.to_string())).expect(&*("get_col: No column called ".to_string() + col))
        }

        pub fn get_f64_col(&self, col: &str) -> &FloatCol {
            self.f64_columns.get(&(col.to_string())).expect(&*("get_col: No column called ".to_string() + col))
        }

        pub fn get_indices(&self, indices: Iter<usize>) -> Table {
            let mut selection_i64: HashMap<String, CrackableCol> = HashMap::new();
            for (name, col) in &self.i64_columns {
                let mut v_buffer = Vec::with_capacity(indices.len());
                for &i in indices.clone() {
                    v_buffer.push(col.v[i]);
                }
                let mut c_buffer = CrackableCol::empty();
                c_buffer.v = v_buffer;
                selection_i64.insert(name.clone(), c_buffer);
            }
            let mut selection_f64: HashMap<String, FloatCol> = HashMap::new();
            for (name, col) in &self.f64_columns {
                let mut v_buffer = Vec::with_capacity(indices.len());
                for &i in indices.clone() {
                    v_buffer.push(col.v[i]);
                }
                let mut c_buffer = FloatCol::empty();
                c_buffer.v = v_buffer;
                selection_f64.insert(name.clone(), c_buffer);
            }

            let mut t = Table::new();
            t.i64_columns = selection_i64;
            t.f64_columns = selection_f64;
            t.count = indices.len();

            let mut indexed_crk_v = Vec::with_capacity(indices.len());
            if self.crk_col.crk.len() > 0 {
                let mut indexed_crk_col = Vec::with_capacity(indices.len());
                for &i in indices.clone() {
                    indexed_crk_col.push(self.crk_col.crk[i]);
                    indexed_crk_v.push(self.crk_col.v[i]);
                }
                t.crk_col.crk     = indexed_crk_col;
                t.crk_col.crk_idx = CrackerIndex::new();
            } else {
                for &i in indices.clone() {
                    indexed_crk_v.push(self.crk_col.v[i]);
                }
            }
            t.crk_col.v = indexed_crk_v;
            t
        }

        // Returns the elements of T where the cracker columns's value equals X
        pub fn cracker_select_specific(&mut self, x: i64) -> Table {
            self.cracker_select_in_three(x, x, true, true)
        }

        // Returns the elements of T where the cracker columns's value is between LOW and HIGH, with inclusivity given by INC_L and INC_H.
        pub fn cracker_select_in_three(&mut self, low: i64, high: i64, inc_l: bool, inc_h: bool) -> Table {
            let adjusted_low = low + !inc_l as i64;
            let adjusted_high = high - !inc_h as i64;
            // c_low(x)  <=> x outside catchment at low  end
            // c_high(x) <=> x outside catchment at high end
            #[inline] let c_low = |x| x < adjusted_low;
            #[inline] let c_high = |x| x > adjusted_high;

            // Start with a pointer at both ends of the piece: p_low, p_high
            let mut p_low =  self.crk_col.crk_idx.lower_bound(&adjusted_low).unwrap_or(0);
            let mut p_high = self.crk_col.crk_idx.upper_bound(&(high + inc_h as i64)).unwrap_or((self.count - 1) as usize);

            let is_uniform_column_piece = adjusted_low == adjusted_high && self.crk_col.crk_idx.contains(adjusted_low) && self.crk_col.crk_idx.contains(adjusted_low + 1);
            if is_uniform_column_piece {
                return self.get_indices(self.crk_col.base_idx[p_low..(p_high + 1)].iter());
            }

            // while p_low is pointing at an element satisfying c_low,  move it forwards
            while c_low(self.crk_col.crk[p_low]) {
                p_low += 1;
                if p_low == self.count as usize {
                    return self.get_indices(self.crk_col.base_idx[0..0].iter());
                }
            }

            // while p_high is pointing at an element satisfying c_high, move it backwards
            while c_high(self.crk_col.crk[p_high]) {
                p_high -= 1;
                if p_high == 0 && c_high(self.crk_col.crk[p_high]) {
                    return self.get_indices(self.crk_col.base_idx[0..0].iter());
                }
            }

            if p_low == p_high {
                return self.get_indices(self.crk_col.base_idx[p_low..(p_high + 1)].iter());
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

            let high_ptr = if p_itr >= self.count { self.count - 1 } else { p_itr };
            self.crk_col.crk_idx.insert(low + !inc_l as i64, p_low);
            self.crk_col.crk_idx.insert(high + inc_h as i64, high_ptr);
            self.get_indices(self.crk_col.base_idx[p_low..p_itr].iter())
        }

        // Returns the elements of T where the cracker column's value is less than MED, with inclusivity given by INC
        pub fn cracker_select_in_two(&mut self, med: i64, inc: bool) -> Table {
            let adjusted_med  = med + inc as i64;
            // cond(x) returns x inside catchment
            #[inline] let cond = |x| x < adjusted_med;

            // Start with pointers at the start and end of the array
            let initial_p_low  = 0;
            let mut p_high = self.crk_col.crk_idx.upper_bound(&adjusted_med).unwrap_or((self.count - 1) as usize);

            // Save p_low for later:
            let mut p_low = initial_p_low.clone();

            // while p_low is pointing at an element already in the catchment, move it forwards
            while cond(self.crk_col.crk[p_low]) {
                p_low += 1;
                if p_low == self.count as usize {
                    return self.get_indices(self.crk_col.base_idx.iter());
                }
            }

            // while p_high is pointing at an element already outside the catchment, move it backwards
            while !cond(self.crk_col.crk[p_high]) {
                p_high -= 1;
                if p_high == 0 && !cond(self.crk_col.crk[p_high]) {
                    return self.get_indices(self.crk_col.base_idx[0..0].iter());
                }
            }

            // At this point, !cond(col[p_low]) && cond(col[p_high])
            while p_low <= p_high {
                self.crk_col.crk.swap(p_low, p_high);
                self.crk_col.base_idx.swap(p_low, p_high);
                while cond(self.crk_col.crk[p_low]) {
                    p_low += 1;
                }
                while !cond(self.crk_col.crk[p_high]) {
                    p_high -= 1;
                }
            }
            self.crk_col.crk_idx.insert(adjusted_med, p_low);
            self.get_indices(self.crk_col.base_idx[initial_p_low..p_low].iter())
        }

        // Counts the places where a given column equals a given value
        pub fn count_col_eq(&self, col: &str, eq: i64) -> i64 {
            self.get_i64_col(col).v.iter().map(|&x|(x==eq)as i64).fold(0,|sum,x|sum+x) as i64
        }
    }
}

#[cfg(test)]
mod test;
