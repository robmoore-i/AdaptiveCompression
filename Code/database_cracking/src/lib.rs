// All credit for this beautiful AVL tree implementation belongs to: https://github.com/eqv/avl_tree
// What a legend
// I added functions for max_before to go with min_after, and also added functions to the Tree struct
// to go with them. The min_after layout changed a bit, but the underlying algorithm is the same.
pub mod avl {
    use std::cmp;
    use std::cmp::Ordering;

    #[derive(Clone)]
    pub struct Node<K: Ord, D> {
        key: K,
        data: D,
        height: u64,
        left:  Option<Box<Node<K,D>>>,
        right: Option<Box<Node<K,D>>>,
    }

    impl<K:Ord, D> Node<K,D> {
        pub fn new(key: K, data: D) -> Node<K,D> {
            Node::<K,D>{key: key, data: data, height: 1, left: None, right: None}
        }
    }
    
    fn height<K:Ord,D>(node: &Option<Box<Node<K,D>>>) -> u64  {
        return node.as_ref().map_or(0, |succ| succ.height)
    }
    
    // Perform a single right rotation on this (sub) tree
    fn rotate_right<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let mut new_root_box = root.left.take().expect("AVL broken");
        root.left = new_root_box.right.take();
        update_height(&mut root);
        new_root_box.right = Some(root);
        update_height(&mut new_root_box);
        return new_root_box
    }

    // Perform a single left rotation on this (sub) tree
    fn rotate_left<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let mut new_root_box = root.right.take().expect("AVL broken");
        root.right = new_root_box.left.take();
        update_height(&mut root);
        new_root_box.left = Some(root);
        update_height(&mut new_root_box);
        return new_root_box
    }

    // Performs a rotation that counteracts the fact that the left successor is too high
    fn rotate_left_successor<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
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
    fn rotate_right_successor<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
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

    fn diff_of_successors_height<K:Ord,D>(root: &Box<Node<K,D>>) -> i32 {
        let l = height(&root.left);
        let r = height(&root.right);
        (l as i32) - (r as i32)
    }


    // Apply all necessary rotations on root.
    fn rotate_if_necessary<K:Ord,D>(root: Box<Node<K,D>>) -> Box<Node<K,D>> {
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
    fn update_height<K:Ord,D>(root: &mut Node<K,D>) {
        root.height = cmp::max( height(&root.left), height(&root.right) )+1;
    }

    // Recursively insert the (key,data) pair into the given optional succesor and return its new value
    fn insert_in_successor<K:Ord,D>(key: K, data: D, successor: Option<Box<Node<K,D>>>) -> Option<Box<Node<K,D>>> {
                Some(match successor {
                    Some(succ) => insert(key, data, succ),
                    None       => Box::new(Node::new(key, data))
                })
    }

    // Inserts the given data under the key in the tree root. It will replace old data stored
    // under this key if it was allready used in the tree. The resulting tree will be returned
    // (its root may now differ due to rotations, thus the old root is moved into the function)
    pub fn insert<K:Ord,D>(key: K, data: D, mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        match root.key.cmp(&key) {
            Ordering::Equal   => { root.data  = data; return root },
            Ordering::Less    => root.right = insert_in_successor(key, data, root.right.take()),
            Ordering::Greater => root.left  = insert_in_successor(key,data, root.left.take())
        }
        update_height(&mut *root);
        return rotate_if_necessary(root)
    }

    // Returns a read only reference to the data stored under key in the tree given by root
    pub fn search<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<&'a D> {
        search_pair(key,root).map(|(_,v)| v )
    }

    // Returns a read only reference paie to the data stored under key in the tree given by root
    pub fn search_pair<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
        match root.key.cmp(key) {
            Ordering::Equal   => Some((&root.key, &root.data)),
            Ordering::Less    => root.right.as_ref().map_or(None, |succ| search_pair(key, succ)),
            Ordering::Greater => root.left.as_ref().map_or(None, |succ| search_pair(key, succ))
        }
    }

    // Returns the smallest key >= given key.
    pub fn min_after<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
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
    
    // Returns the smallest key >= given key.
    pub fn max_before<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
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
    pub fn min_pair<K:Ord,D>(root: &Box<Node<K,D>>) -> (&K,&D) {
        root.left.as_ref().map_or((&root.key,&root.data), min_pair)
    }

    // Returns the maximal key,value pair within this tree
    pub fn max_pair<K:Ord,D>(root: &Box<Node<K,D>>) -> (&K,&D) {
        root.right.as_ref().map_or((&root.key,&root.data), max_pair)
    }

    // Returns the minimal value within this tree
    pub fn min<K:Ord,D>(root: &Box<Node<K,D>>) -> &D {
        root.left.as_ref().map_or(&root.data, min)
    }

    // Returns the minimal value within this tree
    pub fn max<K:Ord,D>(root: &Box<Node<K,D>>) -> &D {
        root.right.as_ref().map_or(&root.data, max)
    }

    // Will update_heights and rotate the node if necessary, returns the rotated node
    fn updated_node<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>> {
        update_height(&mut root);
        rotate_if_necessary(root)
    }

    // Performs recursive `drop_and_get_min` if a left  since a successor is available
    fn drop_min_from_left<K:Ord,D>(mut root : Box<Node<K,D>>, left: Box<Node<K,D>>) -> (Option<Box<Node<K,D>>>,Box<Node<K,D>>) {
        let (new_left, min) = drop_min(left);
        root.left = new_left;
        (Some(updated_node(root)),min)
    }

    // Finds the minimal value below root and returns a new (optional) tree where the minimal value has been
    // removed and the (optional) minimal node as tuple (new_tree, min);
    fn drop_min<K:Ord,D>(mut root: Box<Node<K,D>>) -> (Option<Box<Node<K,D>>>, Box<Node<K,D>>) {
        match root.left.take() {
            Some(left) => drop_min_from_left(root, left),
            None => (root.right.take(), root)
        }
    }

    // Return a new AVL tree, as the combination of two subtrees with max(l) <= min(r)
    fn combine_two_subtrees<K:Ord,D>(l: Box<Node<K,D>>, r: Box<Node<K,D>>) -> Box<Node<K,D>> {
        let (remaining_tree, min) = drop_min(r);
        let mut new_root = min;
        new_root.left = Some(l);
        new_root.right = remaining_tree;
        updated_node(new_root)
    }

    // Return a new AVL tree, where the root has been removed
    fn delete_root<K:Ord,D>(mut root: Box<Node<K,D>>) -> Option<Box<Node<K,D>>> {
        match ( root.left.take(), root.right.take() ) {
            ( None,     None)    => None,
            ( Some(l),  None)    => Some(l),
            ( None,     Some(r)) => Some(r),
            ( Some(l),  Some(r)) => Some(combine_two_subtrees(l,r))
        }
    }

    // Deletes `key` from the tree `root`. Returns either `Some` tree or if the resilting tree is
    // empty: None.
    pub fn delete<K:Ord,D>(key: K, mut root: Box<Node<K,D>>) -> Option<Box<Node<K,D>>> {
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
    pub struct AVLTree<K:Ord+Copy,D> {
        pub root: Option<Box<Node<K,D>>>
    }

    impl <K:Ord+Copy,D> AVLTree<K,D> {
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

// Implementation of Database
pub mod db {
    use avl::*;
    use std::collections::HashMap;
    use std::slice::Iter;

    #[derive(Clone)]
    pub struct Col<T:Ord+Copy> {
        // Original
        pub v: Vec<T>,
        // Cracked
        pub crk: Vec<T>,
        // Cracker index - for a value v, stores the index p such that
        // for all i < p: c[i] < v. That is - Every value before p in the column
        // is less than v.
        pub crk_idx: AVLTree<T, usize>,
        // Base index - maintains an index into the base columns of the table for alignment
        // during tuple reconstruction.
        pub base_idx: Vec<usize>,
    }

    impl <T:Ord+Copy> Col<T> {
        pub fn empty() -> Col<T> {
            Col {
                v: Vec::new(),
                crk:Vec::new(),
                crk_idx: AVLTree::new(),
                base_idx: Vec::new()
            }
        }
    }

    #[derive(Clone)]
    pub struct Table {
        pub count: usize,
        pub crk_col: Col<i64>,
        pub columns: HashMap<String, Col<i64>>,
    }
    
    impl Table {
        pub fn new() -> Table {
            Table { count: 0, crk_col: Col::empty(), columns: HashMap::new() }
        }
        
        pub fn new_columns(&mut self, col_names: Vec<String>) {
            for col in col_names {
                self.columns.insert(col, Col::empty());
            }
        }

        pub fn standard_insert(&mut self, v: &mut Vec<i64>) {
            let n = v.len();
            self.crk_col.v.append(v);
            self.count += n;
        }
    
        pub fn standard_select<F>(&self, constraint: F) -> Vec<i64>
            where F: Fn(i64) -> bool {
            // Specifying max capacity prevents reallocation.
            let n = {
                self.crk_col.v.len()
            };
            let mut selection = Vec::with_capacity(n);
            for x in &self.crk_col.v {
                if constraint(*x) {
                    selection.push(*x);
                }
            }
            selection
        }

        pub fn set_crk_col(&mut self, col_name: String) {
            let col = match self.columns.get(&col_name) {
                Some(ref c) => *c,
                None        => panic!("set_crk_col: no such col"),
            };
            self.crk_col = col.clone();
        }

        // TODO: Improve exception handling in this function
        pub fn insert(&mut self, new_values: &mut HashMap<String, Vec<i64>>) {
            let mut n_new_tuples = 0;
            for (key, val) in self.columns.iter_mut() {
                let new_elems = new_values.get_mut(key).expect("insert: new_values doesn't have values for all columns");
                let n = new_elems.len();
                if n_new_tuples == 0 || n_new_tuples == n {
                    val.v.append(new_elems);
                    n_new_tuples = n;
                } else {
                    panic!("insert: new_values has vectors of differing lengths");
                }
            }
            self.count += n_new_tuples;
        }

        pub fn get_col(&self, col: String) -> Option<&Col<i64>> {
            self.columns.get(&col)
        }

        pub fn get_indices(&self, indices: Iter<usize>) -> Table {
            let mut clone = self.clone(); // TODO: Performance issue in this clone
            for col in clone.columns.values_mut() {
                let mut indexed_v = Vec::with_capacity(indices.len());
                for &i in indices.clone() {
                    indexed_v.push(col.v[i]);
                }
                col.v = indexed_v;
                println!("col.v:{:?}", col.v);
            }

            let mut indexed_crk_v   = Vec::with_capacity(indices.len());
            if self.crk_col.crk.len() > 0 {
                let mut indexed_crk_col = Vec::with_capacity(indices.len());
                for &i in indices.clone() {
                    indexed_crk_col.push(self.crk_col.crk[i]);
                    indexed_crk_v.push(self.crk_col.v[i]);
                }
                clone.crk_col.crk     = indexed_crk_col;
                clone.crk_col.crk_idx = AVLTree::new();
            } else {
                for &i in indices.clone() {
                    indexed_crk_v.push(self.crk_col.v[i]);
                }
            }
            clone.crk_col.v = indexed_crk_v;
            clone
        }

        pub fn select_in_two(&self, col: String, strictly_less_than: i64) -> Table {
            let candidate_col = self.get_col(col);
            match candidate_col {
                Some(ref c) => {
                    let mut index = Vec::with_capacity(self.count);
                    for i in 0..self.count {
                        if c.v[i] < strictly_less_than {
                            index.push(i);
                        }
                    }
                    self.get_indices(index.iter())
                }
                None => panic!("select_in_two: No such column"), // TODO: Improve exception handling here
            }
        }

        // Returns indices of the base columns such that crk_col's values are between LOW and HIGH
        // with inclusivity determined by INC_L and INC_H.
        pub fn cracker_select_in_three(&mut self, low: i64, high: i64, inc_l: bool, inc_h: bool) -> Table {
            // If column hasn't been cracked before, copy it, and copy a reference to the current
            // indices of the base table.
            if self.crk_col.crk.len() == 0 {
                self.crk_col.crk = self.crk_col.v.clone();
                self.crk_col.base_idx = (0..self.count).collect();
            }

            let adjusted_low  = low  + !inc_l as i64;
            let adjusted_high = high - !inc_h as i64;
            // c_low(x)  <=> x outside catchment at low  end
            // c_high(x) <=> x outside catchment at high end
            #[inline] let c_low =  |x| x < adjusted_low;
            #[inline] let c_high = |x| x > adjusted_high;

            // Start with a pointer at both ends of the array: p_low, p_high
            let mut p_low  = *(self.crk_col.crk_idx.lower_bound(&adjusted_low).unwrap_or(&0));
            let mut p_high = *(self.crk_col.crk_idx.upper_bound(&(high + inc_h as i64)).unwrap_or(&((self.count - 1) as usize)));

            println!("three: Initially:");
            println!("p_low:{}", p_low);
            println!("p_high:{}", p_high);

            // while p_low is pointing at an element satisfying c_low,  move it forwards
            while c_low(self.crk_col.crk[p_low]) {
                p_low += 1;
            }

            // while p_high is pointing at an element satisfying c_high, move it backwards
            while c_high(self.crk_col.crk[p_high]) {
                p_high -= 1;
            }
            let mut p_itr = p_low.clone();
            while p_itr <= p_high {
                if c_low(self.crk_col.crk[p_itr]) {
                    self.crk_col.crk.swap(p_low, p_itr);
                    self.crk_col.base_idx.swap(p_low, p_itr);
                    while c_low(self.crk_col.crk[p_low]) {
                        p_low += 1;
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
            self.crk_col.crk_idx.insert(adjusted_low, p_low);
            self.crk_col.crk_idx.insert(high + !inc_h as i64, p_itr);
            println!("three: Finally:");
            println!("p_low:{}", p_low);
            println!("p_high:{}", p_high);
            self.get_indices(self.crk_col.base_idx[p_low..p_itr].iter())
        }

        // Returns the elements of T less than MED, with inclusivity given by INC
        pub fn cracker_select_in_two(&mut self, med: i64, inc: bool) -> Table {
            // If column hasn't been cracked before, copy it
            if self.crk_col.crk.len() == 0 {
                self.crk_col.crk = self.crk_col.v.clone();
                self.crk_col.base_idx = (0..self.count).collect();
            }

            let adjusted_med  = med + inc as i64;
            // cond(x) returns x inside catchment
            #[inline] let cond = |x| x < adjusted_med;

            // Start with pointers at the start and end of the array
            let mut p_low  = 0;
            let mut p_high = *(self.crk_col.crk_idx.upper_bound(&adjusted_med).unwrap_or(&((self.count - 1) as usize)));

            println!("two: Initially:");
            println!("p_low:{}", p_low);
            println!("p_high:{}", p_high);

            println!("med:{}", med);
            // Save p_low for later:
            let initial_p_low = p_low.clone();

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
                if p_high == 0 {
                    return  self.get_indices(self.crk_col.base_idx[0..0].iter());
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
            println!("two: Finally:");
            println!("p_low:{}", p_low);
            println!("p_high:{}", p_high);
            self.crk_col.crk_idx.insert(adjusted_med, p_low);
            self.get_indices(self.crk_col.base_idx[initial_p_low..p_low].iter())
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use db::*;
    use std::collections::HashMap;

    // I credit these two macros (matches, _tt_as_expr_hack) to this chap:
    // http://rrichardson.github.io/reactor/src/mac/matches.rs.html#18-27
    #[macro_export]
    macro_rules! matches {
        ($expr:expr, $($pat:tt)+) => {
             _tt_as_expr_hack! {
                match $expr {
                    $($pat)+ => true,
                    _        => false
                }
            }
        }
    }
    // Work around "error: unexpected token: `an interpolated tt`", whatever that
    // means. (Probably rust-lang/rust#22819.)
    #[doc(hidden)]
    #[macro_export]
    macro_rules! _tt_as_expr_hack {
        ($value:expr) => ($value)
    }

    fn one_col_test_table() -> Table {
        let mut table = Table::new();
        table.new_columns(vec!["a".to_string()]);
        let mut new_values = HashMap::new();
        new_values.insert("a".to_string(), vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        table.insert(&mut new_values);
        table.set_crk_col("a".to_string());
        table
    }

    #[test]
    fn single_column_table_initialised_empty() {
        let table = Table::new();
        assert_eq!(table.count, 0);
    }
    
    #[test]
    fn standard_insert_to_single_column_table() {
        let mut table = Table::new();
        table.standard_insert(&mut vec![1, 2, 3]);
        assert_eq!(table.count, 3);
        assert_eq!(table.crk_col.v, vec![1, 2, 3]);
    }
    
    #[test]
    fn standard_select_from_single_column_table() {
        let mut table = Table::new();
        table.standard_insert(&mut vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        let selection = table.standard_select(|x| x > 10 && x < 14);
        assert_eq!(selection, vec![13, 12, 11]);
    }

    #[test]
    fn cracker_column_initialised_empty() {
        let table = Table::new();
        assert_eq!(table.crk_col.crk.len(), 0);
    }

    #[test]
    fn cracker_select_in_three_from_single_column_table() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(10, 14, false, false);
            assert_eq!(selection.crk_col.v, vec![13, 12, 11]);
        }
        assert_eq!(table.crk_col.crk, vec![6, 4, 9, 2, 7, 1, 8, 3, 13, 12, 11, 14, 19, 16]);
    }

    #[test]
    fn cracker_select_in_three_can_utilise_previous_queries() {
        let mut table = one_col_test_table();
        {
            table.cracker_select_in_three(10, 14, false, false);
            assert!(table.crk_col.crk_idx.contains(11));
            assert!(table.crk_col.crk_idx.contains(15));
            let selection = table.cracker_select_in_three(5, 10, false, false);
            assert_eq!(selection.crk_col.v, vec![7, 9, 8, 6]);
        }
        assert_eq!(table.crk_col.crk, vec![4, 2, 1, 3, 7, 9, 8, 6, 13, 12, 11, 14, 19, 16]);
    }

    #[test]
    fn cracker_select_in_three_from_single_column_table_inc_low() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(3, 7, true, false);
            assert_eq!(selection.crk_col.v, vec![4, 6, 3]);
        }
        assert_eq!(table.crk_col.crk, vec![1, 2, 4, 6, 3, 12, 7, 9, 19, 16, 14, 11, 8, 13]);
    }

    #[test]
    fn cracker_select_in_three_from_single_column_table_inc_high() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(13, 19, false, true);
            assert_eq!(selection.crk_col.v, vec![19, 16, 14]);
        }
        assert_eq!(table.crk_col.crk, vec![13, 4, 9, 2, 12, 7, 1, 3, 11, 8, 6, 19, 16, 14]);
    }

    #[test]
    fn cracker_select_in_three_from_single_column_table_inc_both() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(1, 6, true, true);
            assert_eq!(selection.crk_col.v, vec![6, 3, 4, 1, 2]);
        }
        assert_eq!(table.crk_col.crk, vec![6, 3, 4, 1, 2, 12, 7, 9, 19, 16, 14, 11, 8, 13]);
    }

    #[test]
    fn cracker_select_in_two_from_single_column_table() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_two(7, true);
            assert_eq!(selection.crk_col.v, vec![6, 3, 4, 1, 2, 7]);
        }
        assert_eq!(table.crk_col.crk, vec![6, 3, 4, 1, 2, 7, 12, 9, 19, 16, 14, 11, 8, 13]);
    }

    #[test]
    fn cracker_select_in_two_from_single_column_table_not_inclusive() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_two(10, false);
            assert_eq!(selection.crk_col.v, vec![6, 8, 4, 9, 2, 3, 7, 1]);
        }
        assert_eq!(table.crk_col.crk, vec![6, 8, 4, 9, 2, 3, 7, 1, 19, 12, 14, 11, 16, 13]);
    }
    
    #[test]
    fn cracker_select_in_two_can_utilise_previous_queries() {
        let mut table = one_col_test_table();
        {
            table.cracker_select_in_three(10, 14, false, false);
            let selection = table.cracker_select_in_two(7, false);
            assert_eq!(selection.crk_col.v, vec![6, 4, 3, 2, 1]);
        }
        assert_eq!(table.crk_col.crk, vec![6, 4, 3, 2, 1, 7, 8, 9, 13, 12, 11, 14, 19, 16]);
    }
    
    #[test]
    fn cracker_select_in_three_after_crack_in_two() {
        let mut table = one_col_test_table();
        {
            table.cracker_select_in_two(7, true);
            let selection = table.cracker_select_in_three(6, 11, true, false);
            assert_eq!(selection.crk_col.v, vec![6, 7, 8, 9]);
        }
        assert_eq!(table.crk_col.crk, vec![3, 4, 1, 2, 6, 7, 8, 9, 19, 16, 14, 11, 12, 13]);
    }
    
    #[test]
    fn crack_in_two_above_upper_limit() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_two(25, true);
            assert_eq!(selection.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        }
        assert_eq!(table.crk_col.crk, [13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
    }
    
    #[test]
    fn crack_in_two_below_lower_limit() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_two(-5, true);
            assert_eq!(selection.crk_col.v, vec![]);
        }
        assert_eq!(table.crk_col.crk, [13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
    }
    
    #[test]
    fn crack_in_three_between_value_within_column_and_above_upper_limit() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(14, 25, true, false);
            assert_eq!(selection.crk_col.v, vec![19, 16, 14]);
        }
        assert_eq!(table.crk_col.crk, [13, 4, 9, 2, 12, 7, 1, 3, 11, 8, 6, 19, 16, 14]);
    }
    
    #[test]
    fn crack_in_three_between_value_within_column_and_below_lower_limit() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(-5, 4, true, false);
            assert_eq!(selection.crk_col.v, vec![3, 1, 2]);
        }
        assert_eq!(table.crk_col.crk, [3, 1, 2, 9, 4, 12, 7, 16, 19, 13, 14, 11, 8, 6]);
    }
    
    #[test]
    fn crack_in_three_select_enture_column() {
        let mut table = one_col_test_table();
        {
            let selection = table.cracker_select_in_three(-50, 200, false, false);
            assert_eq!(selection.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        }
        assert_eq!(table.crk_col.crk, [13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
    }
    
    #[test]
    fn can_crack_in_three_over_three_queries() {
        let mut table = one_col_test_table();
        {
            table.cracker_select_in_three(10, 14, false, false);
            let s1 = table.cracker_select_in_three(3, 11, false, true);
            assert_eq!(s1.crk_col.v, vec![6, 7, 4, 8, 9, 11]);
        }
        {
            let s2 = table.cracker_select_in_three(7, 17, true, false);
            assert_eq!(s2.crk_col.v, vec![7, 8, 9, 11, 12, 13, 14, 16]);
        }
        assert_eq!(table.crk_col.crk, [2, 1, 3, 6, 4, 7, 8, 9, 11, 12, 13, 14, 16, 19]);
    }
    
    #[test]
    fn can_crack_in_two_over_three_queries() {
        let mut table = one_col_test_table();
        {
            let s1 = table.cracker_select_in_two(10, true);
            assert_eq!(s1.crk_col.v, vec![6, 8, 4, 9, 2, 3, 7, 1]);
        }
        {
            let s2 = table.cracker_select_in_two(3, true);
            assert_eq!(s2.crk_col.v, vec![1, 3, 2]);
        }
        {
            let s3 = table.cracker_select_in_two(14, false);
            assert_eq!(s3.crk_col.v, vec![1, 3, 2, 9, 4, 8, 7, 6, 13, 12, 11]);
        }
        assert_eq!(table.crk_col.crk, [1, 3, 2, 9, 4, 8, 7, 6, 13, 12, 11, 14, 16, 19]);
    }
    
    #[test]
    fn cracker_index_handles_inclusivity_at_upper_bound() {
        let mut table = one_col_test_table();
        {
            let s1 = table.cracker_select_in_two(19, true);
            assert_eq!(s1.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        }
        {
            let s2 = table.cracker_select_in_three(10, 19, false, true);
            assert_eq!(s2.crk_col.v, vec![19, 12, 14, 11, 16, 13]);
        }
        assert_eq!(table.crk_col.crk, [4, 9, 2, 7, 1, 3, 8, 6, 19, 12, 14, 11, 16, 13]);
    }
    
    #[test]
    fn cracker_index_handles_inclusivity_close_to_upper_bound() {
        let mut table = one_col_test_table();
        {
            let s1 = table.cracker_select_in_two(19, false);
            assert_eq!(s1.crk_col.v, vec![13, 16, 4, 9, 2, 12, 7, 1, 6, 3, 14, 11, 8]);
        }
        {
            let s2 = table.cracker_select_in_three(10, 19, false, true);
            assert_eq!(s2.crk_col.v, vec![12, 16, 14, 11, 13, 19]);
        }
        assert_eq!(table.crk_col.crk, [4, 9, 2, 7, 1, 6, 3, 8, 12, 16, 14, 11, 13, 19]);
    }
    
    #[test]
    fn cracker_index_handles_inclusivity_at_lower_bound() {
        let mut table = one_col_test_table();
        {
            let s1 = table.cracker_select_in_two(1, true);
            assert_eq!(s1.crk_col.v, vec![1]);
        }
        {
            let s2 = table.cracker_select_in_three(1, 5, true, true);
            assert_eq!(s2.crk_col.v, vec![1, 3, 4, 2]);
        }
        assert_eq!(table.crk_col.crk, [1, 3, 4, 2, 9, 12, 7, 13, 19, 16, 14, 11, 8, 6]);
    }
    
    #[test]
    fn cracker_index_handles_inclusivity_close_to_lower_bound() {
        let mut table = one_col_test_table();
        {
            let s1 = table.cracker_select_in_two(2, false);
            assert_eq!(s1.crk_col.v, vec![1]);
        }
        {
            let s2 = table.cracker_select_in_three(1, 5, true, true);
            assert_eq!(s2.crk_col.v, vec![1, 3, 4, 2]);
        }
        assert_eq!(table.crk_col.crk, [1, 3, 4, 2, 9, 12, 7, 13, 19, 16, 14, 11, 8, 6]);
    }
    
    #[test]
    fn can_create_table_with_three_columns() {
        let mut table = Table::new();
        table.new_columns(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let mut keys = Vec::new();
        for key in table.columns.keys() {
            keys.push(key);
        }
        assert!(keys.contains(&&"a".to_string()));
        assert!(keys.contains(&&"b".to_string()));
        assert!(keys.contains(&&"c".to_string()));
    }

    #[test]
    fn can_insert_into_multi_column_table() {
        let mut table = Table::new();
        table.new_columns(vec!["a".to_string(), "b".to_string()]);
        let mut new_values = HashMap::new();
        new_values.insert("a".to_string(), vec![1, 2, 3]);
        new_values.insert("b".to_string(), vec![4, 5, 6]);
        table.insert(&mut new_values);
        assert!(matches!(table.get_col("a".to_string()), Some(ref _col)));
        assert!(matches!(table.get_col("b".to_string()), Some(ref _col)));
        assert!(matches!(table.get_col("c".to_string()), None));
    }

    fn two_col_test_table() -> Table {
        let mut table = Table::new();
        table.new_columns(vec!["a".to_string(), "b".to_string()]);
        let mut new_values = HashMap::new();
        new_values.insert("a".to_string(), vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        new_values.insert("b".to_string(), vec![1,  1,  0, 0, 0, 1,  0, 0, 1,  0, 1,  1,  0, 0]);
        table.insert(&mut new_values);
        table.set_crk_col("a".to_string());
        table
    }

    #[test]
    fn can_index_into_multi_column_table() {
        let table = two_col_test_table();
        let selection = table.get_indices(vec![0, 1, 5, 8, 10, 11].iter());
        match selection.get_col("a".to_string()) {
            Some(ref col) => assert_eq!(col.v, vec![13, 16, 12, 19, 14, 11]),
            None          => assert!(false),
        }
        match selection.get_col("b".to_string()) {
            Some(ref col) => assert_eq!(col.v, vec![1, 1, 1, 1, 1, 1]),
            None          => assert!(false),
        }
    }

    #[test]
    fn can_select_from_multi_column_table() {
        let table = two_col_test_table();
        let selection = table.select_in_two("a".to_string(), 10);
        let a = selection.get_col("a".to_string());
        let b = selection.get_col("b".to_string());
        match a {
            Some(ref col) => assert_eq!(col.v, vec![4, 9, 2, 7, 1, 3, 8, 6]),
            None          => assert!(false),
        }
        match b {
            Some(ref col) => assert_eq!(col.v, vec![0, 0, 0, 0, 0, 0, 0, 0]),
            None          => assert!(false),
        }
    }

    #[test]
    fn can_set_cracked_column() {
        let table = two_col_test_table();
        match table.get_col("a".to_string()) {
            Some(ref c) => assert_eq!(table.crk_col.v, c.v),
            None        => assert!(false),
        };
    }

    #[test]
    fn crack_returns_indices_into_base_columns() {
        let mut table = two_col_test_table();
        let selection = table.cracker_select_in_three(10, 14, false, false);
        match selection.get_col("a".to_string()) {
            Some(ref c) => assert_eq!(c.v, vec![13, 12, 11]),
            None        => assert!(false),
        };
        match selection.get_col("b".to_string()) {
            Some(ref c) => assert_eq!(c.v, vec![1,  1,  1]),
            None        => assert!(false),
        };
    }
}
