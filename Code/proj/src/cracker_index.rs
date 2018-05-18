use std::cmp;
use std::cmp::Ordering;

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

#[derive(Clone)]
pub struct AVLNode {
    key: i64,
    data: usize,
    height: u64,
    left:  Option<Box<AVLNode>>,
    right: Option<Box<AVLNode>>,
}

impl AVLNode {
    pub fn new(k: i64, d: usize) -> AVLNode {
        AVLNode {key: k, data: d, height: 1, left: None, right: None}
    }
}

fn height (node: &Option<Box<AVLNode>>) -> u64  {
    return node.as_ref().map_or(0, |successor| successor.height)
}

// Perform a single right rotation on this (sub) tree
fn rotate_right(mut root: Box<AVLNode>) -> Box<AVLNode> {
    let mut new_root_box = root.left.take().expect("AVL broken");
    root.left = new_root_box.right.take();
    update_height(&mut root);
    new_root_box.right = Some(root);
    update_height(&mut new_root_box);
    return new_root_box
}

// Perform a single left rotation on this (sub) tree
fn rotate_left(mut root: Box<AVLNode>) -> Box<AVLNode> {
    let mut new_root_box = root.right.take().expect("AVL broken");
    root.right = new_root_box.left.take();
    update_height(&mut root);
    new_root_box.left = Some(root);
    update_height(&mut new_root_box);
    return new_root_box
}

// Performs a rotation that counteracts the fact that the left successor is too high
fn rotate_left_successor(mut root: Box<AVLNode>) -> Box<AVLNode> {
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
fn rotate_right_successor(mut root: Box<AVLNode>) -> Box<AVLNode> {
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

fn diff_of_successors_height(root: &Box<AVLNode>) -> i32 {
    let l = height(&root.left);
    let r = height(&root.right);
    (l as i32) - (r as i32)
}


// Apply all necessary rotations on root.
fn rotate_if_necessary(root: Box<AVLNode>) -> Box<AVLNode> {
    let diff  = diff_of_successors_height(&root);
    if -1 <= diff && diff <= 1 {return root}
    match diff {
        2 => rotate_left_successor(root),
        -2 => rotate_right_successor(root),
        _ => unreachable!()
    }
}

// Update the cached height of root. To call this function make sure that the cached values of
// both children of root ar up to date.
fn update_height(root: &mut AVLNode) {
    root.height = cmp::max( height(&root.left), height(&root.right) )+1;
}

// Recursively insert the (key,data) pair into the given optional succesor and return its new value
fn insert_in_successor(key: i64, data: usize, successor: Option<Box<AVLNode>>) -> Option<Box<AVLNode>> {
    Some(match successor {
        Some(s) => insert(key, data, s),
        None       => Box::new(AVLNode::new(key, data))
    })
}

// Inserts the given data under the key in the tree root. It will replace old data stored
// under this key if it was allready used in the tree. The resulting tree will be returned
// (its root may now differ due to rotations, thus the old root is moved into the function)
pub fn insert(key: i64, data: usize, mut root: Box<AVLNode>) -> Box<AVLNode> {
    match root.key.cmp(&key) {
        Ordering::Equal   => { root.data  = data; return root },
        Ordering::Less    => root.right = insert_in_successor(key, data, root.right.take()),
        Ordering::Greater => root.left  = insert_in_successor(key,data, root.left.take())
    }
    update_height(&mut *root);
    return rotate_if_necessary(root)
}

// Returns a read only reference to the data stored under key in the tree given by root
pub fn search<'a>(key: &i64, root: &'a Box<AVLNode>) -> Option<&'a usize> {
    search_pair(key,root).map(|(_,v)| v )
}

// Returns a read only reference pair to the data stored under key in the tree given by root
pub fn search_pair<'a>(key: &i64, root: &'a Box<AVLNode>) -> Option<(&'a i64,&'a usize)> {
    match root.key.cmp(key) {
        Ordering::Equal   => Some((&root.key, &root.data)),
        Ordering::Less    => root.right.as_ref().map_or(None, |succ| search_pair(key, succ)),
        Ordering::Greater => root.left.as_ref().map_or(None, |succ| search_pair(key, succ))
    }
}

// Returns the smallest key value pair (k, v) s.t. k >= given key.
pub fn min_after<'a>(key: &i64, root: &'a Box<AVLNode>) -> Option<(&'a i64,&'a usize)> {
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
pub fn max_before<'a>(key: &i64, root: &'a Box<AVLNode>) -> Option<(&'a i64,&'a usize)> {
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
pub fn min_pair(root: &Box<AVLNode>) -> (&i64,&usize) {
    root.left.as_ref().map_or((&root.key,&root.data), min_pair)
}

// Returns the maximal key,value pair within this tree
pub fn max_pair(root: &Box<AVLNode>) -> (&i64,&usize) {
    root.right.as_ref().map_or((&root.key,&root.data), max_pair)
}

// Returns the minimal value within this tree
pub fn min(root: &Box<AVLNode>) -> &usize {
    root.left.as_ref().map_or(&root.data, min)
}

// Returns the minimal value within this tree
pub fn max(root: &Box<AVLNode>) -> &usize {
    root.right.as_ref().map_or(&root.data, max)
}

// Will update_heights and rotate the node if necessary, returns the rotated node
fn updated_node(mut root: Box<AVLNode>) -> Box<AVLNode> {
    update_height(&mut root);
    rotate_if_necessary(root)
}

// Performs recursive `drop_and_get_min` if a left  since a successor is available
fn drop_min_from_left(mut root : Box<AVLNode>, left: Box<AVLNode>) -> (Option<Box<AVLNode>>,Box<AVLNode>) {
    let (new_left, min) = drop_min(left);
    root.left = new_left;
    (Some(updated_node(root)),min)
}

// Finds the minimal value below root and returns a new (optional) tree where the minimal value has been
// removed and the (optional) minimal node as tuple (new_tree, min);
fn drop_min(mut root: Box<AVLNode>) -> (Option<Box<AVLNode>>, Box<AVLNode>) {
    match root.left.take() {
        Some(left) => drop_min_from_left(root, left),
        None => (root.right.take(), root)
    }
}

// Return a new AVL tree, as the combination of two subtrees with max(l) <= min(r)
fn combine_two_subtrees(l: Box<AVLNode>, r: Box<AVLNode>) -> Box<AVLNode> {
    let (remaining_tree, min) = drop_min(r);
    let mut new_root = min;
    new_root.left = Some(l);
    new_root.right = remaining_tree;
    updated_node(new_root)
}

// Return a new AVL tree, where the root has been removed
fn delete_root(mut root: Box<AVLNode>) -> Option<Box<AVLNode>> {
    match ( root.left.take(), root.right.take() ) {
        ( None,     None)    => None,
        ( Some(l),  None)    => Some(l),
        ( None,     Some(r)) => Some(r),
        ( Some(l),  Some(r)) => Some(combine_two_subtrees(l,r))
    }
}

// Deletes `key` from the tree `root`. Returns either `Some` tree or None
pub fn delete(key: i64, mut root: Box<AVLNode>) -> Option<Box<AVLNode>> {
    match root.key.cmp(&key) {
        Ordering::Equal =>  return delete_root(root),
        Ordering::Less  => {
            if let Some(successor) = root.right.take() {
                root.right = delete(key, successor);
                return Some(updated_node(root))
            }
        },
        Ordering::Greater => {
            if let Some(successor) = root.left.take() {
                root.left = delete(key, successor);
                return Some(updated_node(root))
            }
        }
    }
    return Some(root);
}

// For all keys > THRESHOLD, subtract their value by AMOUNT.
// Assumed that threshold is a key in the current index
pub fn subtract_where_greater_than(threshold: i64, amount: usize, root: &mut Box<AVLNode>) {
    if root.key > threshold {
        root.data -= amount;
        root.left.as_mut().map(|t| subtract_where_greater_than(threshold, amount, t));
        root.right.as_mut().map(|t| subtract_where_greater_than(threshold, amount, t));
    } else if root.right.is_some() {
        root.right.as_mut().map(|t| subtract_where_greater_than(threshold, amount, t));
    }
}

pub fn print_nodes(root: &Box<AVLNode>) {
    print!("{} -> {} | ", root.key, root.data);
    root.left.as_ref().map(|t| print_nodes(&t));
    root.right.as_ref().map(|t| print_nodes(&t));
}

#[derive(Clone)]
pub struct AVLCrackerIndex {
    pub root: Option<Box<AVLNode>>
}

impl AVLCrackerIndex {
    pub fn new() -> AVLCrackerIndex {
        AVLCrackerIndex{root: None}
    }

    pub fn insert(&mut self, key: i64, data: usize) {
        match self.root.take() {
            Some(box_to_node) => self.root = Some(insert(key, data, box_to_node)),
            None              => self.root = Some(Box::new(AVLNode::new(key,data))),
        }
    }

    pub fn delete(&mut self, key: i64) {
        match self.root.take() {
            Some(box_to_node) => self.root = delete(key,box_to_node),
            None              => return
        }
    }

    pub fn get(&self, key: i64) -> Option<usize> {
        match self.root {
            Some(ref box_to_node) => {
                match search(&key, box_to_node) {
                    Some(node) => Some(*node),
                    None           => None
                }
            },
            None                  => None
        }
    }

    pub fn get_or<'a>(&'a self, key: i64, default: usize) -> usize {
        self.get(key).map_or(default, |data| data)
    }

    pub fn contains(&self, key: i64) -> bool {
        self.get(key).is_some()
    }

    pub fn empty(&self) -> bool { self.root.is_none() }

    // Returns the smallest key >= key
    pub fn upper_bound(&self, key: &i64) -> Option<usize> {
        match self.root {
            Some(ref tree) => {
                match min_after(key, tree) {
                    Some((_k, v)) => Some(*v),
                    None          => None
                }
            },
            None => None
        }
    }

    // Returns the largest key <= key
    pub fn lower_bound(&self, key: &i64) -> Option<usize> {
        match self.root {
            Some(ref tree) => {
                match max_before(key, tree) {
                    Some((_k, v)) => Some(*v),
                    None          => None
                }
            },
            None => None
        }
    }

    // For all keys > THRESHOLD, subtract their value by AMOUNT.
    // Assumed that threshold is a key in the current index.
    pub fn subtract_where_greater_than(&mut self, threshold: i64, amount: usize) {
        match self.root {
            Some(ref mut root) => subtract_where_greater_than(threshold, amount, root),
            None => {}
        }
    }

    pub fn print(&self) {
        match self.root {
            Some(ref root) => {
                print!("| ");
                print_nodes(root);
                println!();
            },
            None => {}
        }
    }
}