// All credit for this beautiful AVL tree implementation belongs to: https://github.com/eqv/avl_tree
// What a legend
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
        pub fn new(key: K, data: D) -> Node<K,D>{
            Node::<K,D>{key: key, data: data, height: 1, left: None, right: None}
        }
    }
    
    fn height<K:Ord,D>(node: &Option<Box<Node<K,D>>>) -> u64  {
        return node.as_ref().map_or(0, |succ| succ.height)
    }
    
    // Perform a single right rotation on this (sub) tree
    fn rotate_right<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>>{
        let mut new_root_box = root.left.take().expect("AVL broken");
        root.left = new_root_box.right.take();
        update_height(&mut root);
        new_root_box.right = Some(root);
        update_height(&mut new_root_box);
        return new_root_box
    }

    // Perform a single left rotation on this (sub) tree
    fn rotate_left<K:Ord,D>(mut root: Box<Node<K,D>>) -> Box<Node<K,D>>{
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
        match diff{
            2 => rotate_left_successor::<K,D>(root),
            -2 => rotate_right_successor::<K,D>(root),
            _ => unreachable!()
        }
    }

    // Update the cached height of root. To call this function make sure that the cached values of
    // both children of root ar up to date.
    fn update_height<K:Ord,D>(root: &mut Node<K,D>){
        root.height = cmp::max( height(&root.left), height(&root.right) )+1;
    }

    // Recursively insert the (key,data) pair into the given optional succesor and return its new value
    fn insert_in_successor<K:Ord,D>(key: K, data: D, successor: Option<Box<Node<K,D>>>)->Option<Box<Node<K,D>>> {
                Some(match successor {
                    Some(succ) => insert(key, data, succ),
                    None =>Box::new(Node::new(key, data))
                })
    }

    // Inserts the given data under the key in the tree root. It will replace old data stored
    // under this key if it was allready used in the tree. The resulting tree will be returned
    // (its root may now differ due to rotations, thus the old root is moved into the function)
    pub fn insert<K:Ord,D>(key: K, data: D, mut root: Box<Node<K,D>>) -> Box<Node<K,D>>{
        match root.key.cmp(&key) {
            Ordering::Equal => { root.data  = data; return root },
            Ordering::Less =>    root.right = insert_in_successor(key, data, root.right.take()),
            Ordering::Greater => root.left  = insert_in_successor(key,data, root.left.take())
        }
        update_height(&mut *root);
        return rotate_if_necessary(root)
    }

    // Returns a read only reference to the data stored under key in the tree given by root
    pub fn search<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<&'a D>{
        search_pair(key,root).map(|(_,v)| v )
    }

    // Returns a read only reference paie to the data stored under key in the tree given by root
    pub fn search_pair<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)>{
        match root.key.cmp(key) {
            Ordering::Equal => Some((&root.key, &root.data)),
            Ordering::Less => root.right.as_ref().map_or(None, |succ| search_pair(key, succ)),
            Ordering::Greater => root.left.as_ref().map_or(None, |succ| search_pair(key, succ))
        }
    }

    // Returns the smallest key and value after the given key.
    pub fn min_after<'a, K:Ord,D>(key: &K, root: &'a Box<Node<K,D>>) -> Option<(&'a K,&'a D)> {
        match root.key.cmp(key){
            Ordering::Equal =>  root.right.as_ref().map_or(None, |succ| Some(min_pair(succ))),
            Ordering::Less =>   root.right.as_ref().map_or(None, |succ| min_after(key, succ)),
            Ordering::Greater => {
                match root.left {
                    Some(ref succ) => min_after(key, &succ).or( Some((&root.key,&root.data)) ),
                    None => Some((&root.key, &root.data))
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
        let (new_left, min) =  drop_min(left);
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
    fn combine_two_subtrees<K:Ord,D>(l: Box<Node<K,D>>, r: Box<Node<K,D>>) -> Box<Node<K,D>>{
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
    pub fn delete<K:Ord,D>(key: K, mut root: Box<Node<K,D>>) -> Option<Box<Node<K,D>>>{
        match root.key.cmp(&key){
            Ordering::Equal =>  return delete_root(root),
            Ordering::Less => {
                if let Some(succ) = root.right.take() {
                    root.right = delete(key, succ);
                    return Some(updated_node(root))
                }
            },
            Ordering::Greater => {
                if let Some(succ) = root.left.take() {
                    root.left =  delete(key, succ);
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
        pub fn new() -> AVLTree<K,D>{
            AVLTree{root: None}
        }

        pub fn insert(&mut self, key: K, data: D) {
            match self.root.take() {
                Some(box_to_node) => self.root = Some(insert::<K,D>(key, data, box_to_node)),
                None => self.root = Some(Box::new(Node::new(key,data))),
            }
        }

        pub fn delete(&mut self, key: K){
            match self.root.take() {
                Some(box_to_node) => self.root = delete(key,box_to_node),
                None => return
            }
        }

        pub fn get(&self, key: K) -> Option<&D>{
            match self.root {
                Some(ref box_to_node) =>search(&key, box_to_node),
                None => None
            }
        }

        pub fn get_or<'a>(&'a self, key: K, default: &'a D) -> &D{
            self.get(key).map_or(default, |data| data)
        }

        pub fn contains(&self, key: K) -> bool {
            self.get(key).is_some()
        }

        pub fn empty(&self) -> bool { self.root.is_none() }
    }
}

// Implementation of Database
pub mod db {
    use avl::*;

    #[derive(Clone)]
    pub struct Table {
        // Meta
        pub index: Vec<u64>,
        pub count: u64,

        // Columns
        pub a: Col,
    }
    
    #[derive(Clone)]
    pub struct Col {
        // Original
        pub v: Vec<i64>,
        // Cracked
        pub crk: Vec<i64>,
        // Cracker index - for a value v, stores a position p and inclusivity inc
        pub crk_idx: AVLTree<i64, usize>,
    }
    
    pub fn new_table() -> Table {
        Table {index: Vec::new(), count: 0, a: Col {v: Vec::new(), crk: Vec::new(), crk_idx: AVLTree {root: None}}}
    }
    
    pub fn standard_insert(t: &mut Table, v: &mut Vec<i64>) {
        let n = v.len() as u64;
        let mut indices: Vec<u64> = (t.count..(t.count + n)).collect();
        t.index.append(&mut indices);
        t.a.v.append(v);
        t.count += n;
    }
    
    pub fn standard_select<F>(t: Table, constraint: F) -> Vec<i64>
        where F: Fn(i64) -> bool {
        let c = t.a;
        // Specifying max capacity prevents reallocation.
        let mut selection = Vec::with_capacity(c.v.len());
        for x in c.v {
            if constraint(x) {
                selection.push(x);
            }
        }
        selection
    }
    
    fn crack_in_three(c: &mut Vec<i64>, pos_l: usize, pos_h: usize, low: i64, high: i64) -> (usize, usize) {
        let mut x1 = pos_l;
        let mut x2 = pos_h;
        println!("");
        println!("Cracking between {} and {} in the range {} to {}", pos_l, pos_h, low, high);
        while c[x2] >= high && x2 > x1 {
            x2 -= 1;            
        }
        println!("Moved x2 to {} (value {})", x2, c[x2]);
        let mut x3 = x2.clone();
        while c[x3] > low && x3 > x1 {
            if c[x3] >= high {
                println!("Pass 1: swapping {} and {}", c[x2], c[x3]);
                c.swap(x2, x3);
                x2 -= 1;
            }
            x3 -= 1;
        }
        println!("After pass 1, x = ({}, {}, {})", x1, x2, x3);
        while x1 <= x3 {
            if c[x1] < low {
                x1 += 1;
            } else {
                println!("Pass 2a: swapping {} and {}", c[x1], c[x3]);
                c.swap(x1, x3);
                while c[x3] > low && x3 > x1 {
                    if c[x3] >= high {
                        println!("Pass 2b: swapping {} and {}", c[x2], c[x3]);
                        c.swap(x2, x3);
                        x2 -= 1;
                    }
                    x3 -= 1;
                }
            }     
        }
        println!("  orig: `{:?}", vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        (x1, x2 + 1)
    }
    
    // Selects from T between POS_L and POS_H values strictly in between LOW and HIGH
    pub fn cracker_select_in_three(t: &mut Table, pos_l: usize, pos_h: usize, low: i64, high: i64) -> &[i64] {
        if t.a.crk.len() == 0 {
            t.a.crk = t.a.v.clone();
        }
        let idx_l = *t.a.crk_idx.get_or(low,  &pos_l);
        let idx_h = *t.a.crk_idx.get_or(high, &pos_h);
        if idx_l != pos_l && idx_h != pos_h {
            return &t.a.crk[idx_l..idx_h];
        }
        let (l, h) = crack_in_three(&mut t.a.crk, pos_l, pos_h, low, high);
        t.a.crk_idx.insert(low,  l);
        t.a.crk_idx.insert(high, h);
        &t.a.crk[l..h]
    }
    
    fn crack_in_two(c: &mut Vec<i64>, pos_l: usize, pos_h: usize, med: i64) -> (usize, usize) {
        let mut x1 = pos_l;
        let mut x2 = pos_h;
        while x1 < x2 {
            if c[x1] < med {
                x1 += 1;
            } else {
                while c[x2] >= med && x2 > x1 {
                    x2 -= 1;
                }
                c.swap(x1, x2);
                x1 += 1;
                x2 -= 1;
            }
        }
        (0, x2 + 1)
    }
    
    // Selects from T between POS_L and POS_H values strictly less than MED
    pub fn cracker_select_in_two(t: &mut Table, pos_l: usize, pos_h: usize, med: i64) -> &[i64] {
        if t.a.crk.len() == 0 {
            t.a.crk = t.a.v.clone();
        }
        let idx_h = *t.a.crk_idx.get_or(med, &pos_h);
        if idx_h != pos_h {
            return &t.a.crk[pos_l..idx_h];
        }
        let (l, h) = crack_in_two(&mut t.a.crk, pos_l, pos_h, med);
        t.a.crk_idx.insert(med, h);
        &t.a.crk[l..h]
    }
}

// Tests
#[cfg(test)]
mod tests {
    use db::*;

    #[test]
    fn single_column_table_initialised_empty() {
        let table = new_table();
        assert_eq!(table.count, 0);
    }
    
    #[test]
    fn standard_insert_to_single_column_table() {
        let mut table = new_table();
        standard_insert(&mut table, &mut vec![1, 2, 3]);
        assert_eq!(table.count, 3);
        assert_eq!(table.index, vec![0, 1, 2]);
        assert_eq!(table.a.v, vec![1, 2, 3]);
    }
    
    #[test]
    fn standard_select_from_single_column_table() {
        let mut table = new_table();
        standard_insert(&mut table, &mut vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
        let selection = standard_select(table, |x| x > 10 && x < 14);
        assert_eq!(selection, vec![13, 12, 11]);
    }
    
    #[test]
    fn cracker_select_in_three_from_single_column_table() {
        let mut table = new_table();
        assert_eq!(table.a.crk.len(), 0);
        {
            standard_insert(&mut table, &mut vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
            let max_pos = (table.count - 1) as usize;
            let selection = cracker_select_in_three(&mut table, 0, max_pos, 10, 14);
            assert_eq!(*selection, [11, 12, 13]);
        }
        let max_pos = (table.count - 1) as usize;
        let selection = cracker_select_in_three(&mut table, 0, max_pos, 10, 14);
        assert_eq!(*selection, [11, 12, 13]);
    }

    #[test]
    fn cracker_select_in_two_from_single_column_table() {
        let mut table = new_table();
        assert_eq!(table.a.crk.len(), 0);
        {
            standard_insert(&mut table, &mut vec![13, 16, 4, 9, 2, 12, 7, 1, 19, 3, 14, 11, 8, 6]);
            let max_pos = (table.count - 1) as usize;
            let selection = cracker_select_in_two(&mut table, 0, max_pos, 7);
            assert_eq!(*selection, [6, 3, 4, 1, 2]);
        }
        let max_pos = (table.count - 1) as usize;
        let selection = cracker_select_in_two(&mut table, 0, max_pos, 7);
        assert_eq!(*selection, [6, 3, 4, 1, 2]);
    }
}
