// B+Tree Implementation with Generics and Binary Search
// December 26, 2025
//
// A learning-focused B+Tree implementation in Rust with:
// - Generic key-value pairs (K, V)
// - Binary search for improved performance
// - All data lives in leaf nodes
// - Internal nodes only store keys for navigation
// - Leaf nodes are linked together (like a linked list)
// - Supports insert, search, and delete with automatic rebalancing

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

/// Main B+Tree structure with generic key and value types
///
/// The `order` determines how many keys a node can hold before splitting.
/// For example, order=3 means a node can have at most 3 keys.
pub struct BTree<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    pub root: Option<Rc<RefCell<BTreeNode<K, V>>>>,
    order: usize,
}

/// A node in the B+Tree - either Internal (for navigation) or Leaf (stores data)
/// Represents either an internal node or a leaf node
///
/// Internal nodes guide searches to the right child:
/// ```text
///     [20 | 40]
///    /    |    \
///  ...   ...   ...
/// ```
///
/// Leaf nodes actually store the data entries:
/// ```text
/// [10, 15, 18] -> [20, 25, 30] -> [40, 45]
///
#[derive(Debug)]
pub enum BTreeNode<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    /// Internal nodes guide searches but don't store data
    Internal {
        parent: Option<Weak<RefCell<BTreeNode<K, V>>>>,
        keys: Vec<K>,
        children: Vec<Rc<RefCell<BTreeNode<K, V>>>>,
    },

    /// Leaf nodes store the actual data entries
    Leaf {
        parent: Option<Weak<RefCell<BTreeNode<K, V>>>>,
        data: Vec<Entry<K, V>>,
        next: Option<Rc<RefCell<BTreeNode<K, V>>>>,
    },
}

/// Helper struct providing mutable access to leaf node fields
#[derive(Debug)]
#[allow(dead_code)]
pub struct LeafNode<'l, K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    pub parent: &'l mut Option<Weak<RefCell<BTreeNode<K, V>>>>,
    pub data: &'l mut Vec<Entry<K, V>>,
    pub next: &'l mut Option<Rc<RefCell<BTreeNode<K, V>>>>,
}

/// A key-value entry stored in leaf nodes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Entry<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    pub key: K,
    pub val: V,
}

/// Comparison ordering for comparing two nodes
#[allow(dead_code)]
pub enum NodeCmpOrd {
    Less,
}

/// Strategy enum for delete operations
pub enum DeletePlanner {
    Empty,
    Simple,
    RightBorrow,
    LeftBorrow,
    Merge,
}

impl<K, V> BTree<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    /// Creates a new empty B+Tree with the specified order
    ///
    /// Example:
    /// ```
    // let mut tree = BTree::new(5); // order 5 means max 5 keys per node    pub fn new(order: usize) -> Self {
    pub fn new(order: usize) -> Self {
        Self { root: None, order }
    }

    // Case 1: No root exists at all
    pub fn _is_empty(&self) -> bool {
        if self.root.is_none() {
            return true;
        }
        // Case 2: Root exists but it's an empty leaf
        // This happens when the tree was created but nothing was inserted
        if let BTreeNode::Leaf { data, .. } = &*self.root.as_ref().unwrap().borrow() {
            return data.is_empty();
        }
        // Case 3: Root is an internal node, so we have data somewhere
        false
    }

    /// Searches for a key in the tree and returns the entry if found
    ///
    /// How it works:
    /// 1. Start at root, follow keys down to correct leaf
    /// 2. Search the leaf for the exact key
    ///
    /// Example tree search for key 25:
    /// ```text
    ///        [30]
    ///       /    \
    ///   [10,20]  [30,40]
    ///      ^
    ///   found here
    /// ```
    pub fn search(&self, key: &K) -> Option<Entry<K, V>> {
        if self.root.is_none() {
            return None;
        }

        let mut current = self.root.as_ref().map(Rc::clone).unwrap();

        // Navigate down to the correct leaf using binary search
        let _ = loop {
            let next = {
                let node = current.borrow();
                match &*node {
                    BTreeNode::Internal { keys, children, .. } => {
                        // Binary search to find the correct child
                        let idx = match keys.binary_search(key) {
                            Ok(i) => i + 1, // Key found, go to right child
                            Err(i) => i,    // Key not found, i is insertion point
                        };
                        Rc::clone(&children[idx])
                    }
                    BTreeNode::Leaf { .. } => {
                        break;
                    }
                }
            };
            current = next;
        };

        // Binary search within the leaf node
        if let BTreeNode::Leaf { data: entries, .. } = &*current.borrow() {
            match entries.binary_search_by(|entry| entry.key.cmp(key)) {
                Ok(idx) => return Some(entries[idx].clone()),
                Err(_) => return None,
            }
        }

        None
    }

    /// Inserts a new entry into the tree
    ///
    /// Process:
    /// 1. Find the correct leaf node
    /// 2. Insert the entry and sort
    /// 3. If node overflows, split it
    /// 4. Splits can cascade up to the root
    ///
    /// Example of splitting (order=3, max 3 keys):
    /// ```text
    /// Before: [10, 20, 30, 40] <- overflow!
    ///
    /// After:       [30]
    ///             /    \
    ///        [10,20]  [30,40]
    /// ```
    pub fn insert(&mut self, entry: Entry<K, V>) {
        let leaf = self.find_leaf(&entry.key);

        if let BTreeNode::Leaf { data: keys, .. } = &mut *leaf.borrow_mut() {
            // Binary search to find insertion position
            match keys.binary_search_by(|e| e.key.cmp(&entry.key)) {
                Ok(idx) => {
                    // Key exists, update the value
                    keys[idx] = entry;
                }
                Err(idx) => {
                    // Key doesn't exist, insert at the correct position
                    keys.insert(idx, entry);
                }
            }
        }

        let is_leaf = (*leaf.borrow()).is_leaf();

        if (*leaf.borrow()).is_node_full(self.order) {
            if is_leaf {
                self.split_leaf(Rc::clone(&leaf));
            }
        }
    }

    /// Walks down the tree to find which leaf should contain this key
    /// Creates a new root leaf if tree is empty
    fn find_leaf(&mut self, key: &K) -> Rc<RefCell<BTreeNode<K, V>>> {
        if self.root.is_none() {
            let leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                parent: None,
                data: Vec::new(),
                next: None,
            }));
            self.root = Some(Rc::clone(&leaf));
            return leaf;
        }

        let mut current = Rc::clone(self.root.as_ref().unwrap());

        let leaf = loop {
            let next = {
                let node = current.borrow();

                match &*node {
                    BTreeNode::Leaf { .. } => {
                        break Rc::clone(&current);
                    }

                    BTreeNode::Internal { keys, children, .. } => {
                        // Binary search to find the correct child
                        let idx = match keys.binary_search(key) {
                            Ok(i) => i + 1,
                            Err(i) => i,
                        };
                        Rc::clone(&children[idx])
                    }
                }
            };

            current = next;
        };

        leaf
    }

    /// Gets or creates a parent node
    /// Used during splits when a node needs a parent to hold the separator key
    fn get_node_parent(
        &mut self,
        parent: &mut Option<Weak<RefCell<BTreeNode<K, V>>>>,
    ) -> Weak<RefCell<BTreeNode<K, V>>> {
        if parent.is_none() {
            let parent = Rc::new(RefCell::new(BTreeNode::new_internal()));
            self.root = Some(parent);
            return self.root.as_ref().map(Rc::downgrade).unwrap();
        }

        ((parent.as_ref().unwrap()).upgrade())
            .as_ref()
            .map(Rc::downgrade)
            .unwrap()
    }
    /// Splits an overflowing leaf node into two nodes
    ///
    /// Process:
    /// ```text
    /// Before: [10, 20, 30, 40] (overflow)
    ///
    /// After:  Parent: [30]
    ///         /           \
    ///    [10, 20]      [30, 40]
    ///       |  ----------->  |  (linked via 'next')
    /// ```
    ///
    /// The middle key gets promoted to parent
    fn split_leaf(&mut self, leaf: Rc<RefCell<BTreeNode<K, V>>>) {
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode<K, V>>>> = None;
        let mut mut_leaf_ptr = leaf.borrow_mut();

        if let BTreeNode::Leaf {
            parent: left_child_parent,
            data: left_keys,
            next,
        } = &mut *mut_leaf_ptr
        {
            let ptr_leaf_parent = (self.get_node_parent(left_child_parent).upgrade()).unwrap();
            let mut leaf_parent = ptr_leaf_parent.borrow_mut();

            if let BTreeNode::Internal {
                keys: parent_keys,
                children: parent_children,
                ..
            } = &mut *leaf_parent
            {
                let middle = left_keys.len() / 2;
                let key_to_promote = left_keys[middle].key.clone();

                let right_child = {
                    let tmp = Rc::new(RefCell::new(BTreeNode::Leaf {
                        parent: Some(Rc::downgrade(&ptr_leaf_parent)),
                        data: left_keys.split_off(middle),
                        next: {
                            let mut next_ptr = None;
                            if next.is_some() {
                                next_ptr = Some(next.as_ref().map(Rc::clone).unwrap());
                            }
                            next_ptr
                        },
                    }));
                    tmp
                };

                *left_child_parent = Some(Rc::downgrade(&ptr_leaf_parent));
                *next = Some(Rc::clone(&right_child));

                if parent_children.is_empty() {
                    parent_children.push(Rc::clone(&leaf));
                    parent_keys.push(key_to_promote);
                    parent_children.push(Rc::clone(&right_child));
                } else {
                    // Binary search to find position
                    let left_index = parent_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &leaf))
                        .unwrap();

                    parent_keys.insert(left_index, key_to_promote);
                    parent_children.insert(left_index + 1, Rc::clone(&right_child));
                }

                if parent_keys.len() >= self.order {
                    overflowed_parent = Some(Rc::clone(&ptr_leaf_parent));
                }
            }
            // Manuall drop to prevent borrowing overhead later
            drop(leaf_parent);
            drop(mut_leaf_ptr);
        }

        if overflowed_parent.is_some() {
            self.split_internal(overflowed_parent.unwrap());
        }
    }

    /// Splits an overflowing internal node
    ///
    /// Similar to leaf split but:
    /// - Middle key moves UP to parent (doesn't stay in either child)
    /// - Children pointers must be redistributed
    /// - All moved children need their parent pointer updated
    fn split_internal(&mut self, node: Rc<RefCell<BTreeNode<K, V>>>) {
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode<K, V>>>> = None;

        let mut mut_left_ptr = node.borrow_mut();
        if let BTreeNode::Internal {
            parent: left_node_parent,
            keys: left_node_keys,
            children: left_node_children,
        } = &mut *mut_left_ptr
        {
            let parent = self.get_node_parent(left_node_parent).upgrade().unwrap();
            let mut mut_parent_ptr = parent.borrow_mut();

            let middle = left_node_keys.len() / 2;
            let key_to_promote = left_node_keys[middle].clone();

            let right_node = Rc::new(RefCell::new(BTreeNode::Internal {
                parent: Some(Rc::downgrade(&parent)),
                keys: {
                    let keys = left_node_keys.split_off(middle + 1);
                    left_node_keys.pop();
                    keys
                },
                children: left_node_children.split_off(middle + 1),
            }));

            *left_node_parent = Some(Rc::downgrade(&parent));

            for child in left_node_children.iter() {
                let mut child_node = child.borrow_mut();
                match &mut *child_node {
                    BTreeNode::Internal {
                        parent: child_parent,
                        ..
                    }
                    | BTreeNode::Leaf {
                        parent: child_parent,
                        ..
                    } => {
                        *child_parent = Some(Rc::downgrade(&node));
                    }
                }
            }

            let mut mut_right_ptr = right_node.borrow_mut();
            if let BTreeNode::Internal {
                children: right_node_children,
                ..
            } = &mut *mut_right_ptr
            {
                for child in right_node_children.iter_mut() {
                    let mut child_node = child.borrow_mut();

                    if let BTreeNode::Internal {
                        parent: child_parent,
                        ..
                    } = &mut *child_node
                    {
                        *child_parent = Some(Rc::downgrade(&right_node));
                    }

                    if let Some(_) = (*child_node).as_mut_leaf(|parent, _, _| {
                        *parent = Some(Rc::downgrade(&right_node));
                    }) {}
                }
            }

            drop(mut_right_ptr);

            if let BTreeNode::Internal {
                keys: parent_keys,
                children: parent_children,
                ..
            } = &mut *mut_parent_ptr
            {
                if parent_children.is_empty() {
                    parent_children.push(Rc::clone(&node));
                    parent_keys.push(key_to_promote);
                    parent_children.push(Rc::clone(&right_node));
                } else {
                    let left_index = parent_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &node))
                        .unwrap();

                    parent_keys.insert(left_index, key_to_promote);
                    parent_children.insert(left_index + 1, Rc::clone(&right_node));
                }

                if parent_keys.len() >= self.order {
                    overflowed_parent = Some(Rc::clone(&parent));
                }
            }

            drop(mut_parent_ptr);
            drop(mut_left_ptr);
        }

        if overflowed_parent.is_some() {
            self.split_internal(overflowed_parent.unwrap());
        }
    }

    /// Deletes a key from the tree
    pub fn delete(&mut self, key: &K) -> bool {
        let leaf_rc = self.find_leaf(key);

        if Rc::ptr_eq(&leaf_rc, self.root.as_ref().unwrap()) {
            let state = leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                match data.binary_search_by(|entry| entry.key.cmp(key)) {
                    Ok(idx) => {
                        data.remove(idx);
                        true
                    }
                    Err(_) => false,
                }
            });

            if state.unwrap() {
                return true;
            } else {
                return false;
            }
        }

        let plan = self.delete_planner(key, Rc::clone(&leaf_rc));

        match plan {
            (DeletePlanner::Empty, ..) => {
                return false;
            }

            (DeletePlanner::Simple, ..) => {
                return leaf_rc
                    .borrow_mut()
                    .as_mut_leaf(|_, data, _| {
                        match data.binary_search_by(|entry| entry.key.cmp(key)) {
                            Ok(idx) => {
                                data.remove(idx);
                                true
                            }
                            Err(_) => false,
                        }
                    })
                    .unwrap_or(false);
            }

            (DeletePlanner::RightBorrow, r_leaf, pos) => {
                return self.right_borrow(Rc::clone(&leaf_rc), key, r_leaf, pos);
            }

            (DeletePlanner::LeftBorrow, left_sibl, left_sibl_pos) => {
                return self.left_borrow(key, leaf_rc, left_sibl, left_sibl_pos);
            }

            _ => {
                return self.merge_leaf(leaf_rc, key);
            }
        }

        false
    }

    /// Decides which deletion strategy to use
    fn delete_planner(
        &mut self,
        _: &K,
        leaf_rc: Rc<RefCell<BTreeNode<K, V>>>,
    ) -> (DeletePlanner, Rc<RefCell<BTreeNode<K, V>>>, usize) {
        if let BTreeNode::Internal { parent, keys, .. } = &*leaf_rc.borrow() {
            if parent.is_none() {
                if keys.is_empty() {
                    return (DeletePlanner::Empty, Rc::clone(&leaf_rc), 0x00);
                }
            }
        }

        let tmp_borrow = leaf_rc.borrow();
        if let BTreeNode::Leaf { parent, data, .. } = &*tmp_borrow {
            if parent.is_none() {
                if data.is_empty() {
                    return (DeletePlanner::Empty, Rc::clone(&leaf_rc), 0x00);
                }
            }
        }
        drop(tmp_borrow);

        let state = leaf_rc.borrow().can_borrow(self.order);
        if state {
            return (DeletePlanner::Simple, Rc::clone(&leaf_rc), 0x00);
        }

        let right_sibl = BTreeNode::right_sibling(Rc::clone(&leaf_rc));
        if right_sibl.is_some() {
            let (next_rc, right_sibling_pos) = right_sibl.unwrap();
            let state = next_rc.borrow().can_borrow(self.order);
            if state {
                return (
                    DeletePlanner::RightBorrow,
                    Rc::clone(&next_rc),
                    right_sibling_pos,
                );
            }
        }

        let left_sibl = BTreeNode::left_sibling(Rc::clone(&leaf_rc));
        if left_sibl.is_some() {
            let (left, pos) = left_sibl.unwrap();
            if left.borrow().can_borrow(self.order) {
                return (DeletePlanner::LeftBorrow, left, pos);
            }
        }

        (DeletePlanner::Merge, Rc::clone(&leaf_rc), 0x03)
    }

    /// Handles delete by borrowing from right sibling
    pub fn right_borrow(
        &mut self,
        leaf: Rc<RefCell<BTreeNode<K, V>>>,
        deleted_key: &K,
        right_sibl: Rc<RefCell<BTreeNode<K, V>>>,
        right_sibling_pos: usize,
    ) -> bool {
        if let BTreeNode::Leaf { parent, data, .. } = &mut *leaf.borrow_mut() {
            match data.binary_search_by(|e| e.key.cmp(deleted_key)) {
                Ok(idx) => {
                    data.remove(idx);
                }
                Err(_) => return false,
            }

            if let BTreeNode::Leaf { data: sib_dat, .. } = &mut *right_sibl.borrow_mut() {
                let entry = sib_dat.remove(0);

                // Binary search to find insertion position
                match data.binary_search_by(|e| e.key.cmp(&entry.key)) {
                    Ok(idx) | Err(idx) => data.insert(idx, entry),
                }

                let new_sep = sib_dat[0].key.clone();

                let prnt = parent.as_ref().unwrap().upgrade().unwrap();
                if let BTreeNode::Internal { keys, .. } = &mut *prnt.borrow_mut() {
                    keys[right_sibling_pos - 1] = new_sep;
                }
            }
        }
        true
    }

    /// Handles delete by borrowing from left sibling
    pub fn left_borrow(
        &mut self,
        key: &K,
        leaf_rc: Rc<RefCell<BTreeNode<K, V>>>,
        left_sibl: Rc<RefCell<BTreeNode<K, V>>>,
        left_sibl_pos: usize,
    ) -> bool {
        if let BTreeNode::Leaf { parent, data, .. } = &mut *leaf_rc.borrow_mut() {
            match data.binary_search_by(|e| e.key.cmp(key)) {
                Ok(idx) => {
                    data.remove(idx);
                }
                Err(_) => return false,
            }

            if let BTreeNode::Leaf { data: left_dat, .. } = &mut *left_sibl.borrow_mut() {
                let last_idx = left_dat.len() - 1;
                let borrowed_entry = left_dat.remove(last_idx);
                let t_key = borrowed_entry.key.clone();

                data.insert(0, borrowed_entry);

                let prnt_tmp = parent.as_ref().unwrap().upgrade().unwrap();
                let mut prnt = prnt_tmp.borrow_mut();
                if let BTreeNode::Internal { keys: pr_k, .. } = &mut *prnt {
                    pr_k[left_sibl_pos] = t_key;
                }
            }
        }

        true
    }

    /// Merges the underflowing node with a sibling
    pub fn merge_leaf(&mut self, leaf_rc: Rc<RefCell<BTreeNode<K, V>>>, key: &K) -> bool {
        let mut underflowed_parent: Option<_> = None;
        let left_sibl = BTreeNode::left_sibling(Rc::clone(&leaf_rc));

        if left_sibl.is_none() {
            let (right_sibl, _) = BTreeNode::right_sibling(Rc::clone(&leaf_rc)).unwrap();
            let mut tmp_r_borrow = right_sibl.borrow_mut();

            if let BTreeNode::Leaf {
                parent: l_prnt,
                data,
                next,
            } = &mut *tmp_r_borrow
            {
                let mut leaf_tmp_brr = leaf_rc.borrow_mut();

                if let BTreeNode::Leaf {
                    data: curr_node_data,
                    next: curr_next,
                    ..
                } = &mut *leaf_tmp_brr
                {
                    match curr_node_data.binary_search_by(|e| e.key.cmp(key)) {
                        Ok(idx) => {
                            curr_node_data.remove(idx);
                        }
                        Err(_) => return false,
                    }

                    curr_node_data.append(data);

                    if next.is_none() {
                        *curr_next = None;
                    } else {
                        *curr_next = Some(next.as_ref().map(Rc::clone).unwrap());
                    }

                    let t_prnt = l_prnt.as_ref().unwrap().upgrade().unwrap();
                    if let BTreeNode::Internal { keys, children, .. } = &mut *t_prnt.borrow_mut() {
                        keys.remove(0);
                        children.remove(1);

                        if (self.order as f32 / 2_f32).ceil() - 1_f32 > keys.len() as f32 {
                            underflowed_parent = Some(Rc::clone(&t_prnt));
                        }
                    }
                    drop(t_prnt);
                }
                drop(leaf_tmp_brr);
            }
            drop(tmp_r_borrow);

            if underflowed_parent.is_some() {
                self.fix_parent_underflow(underflowed_parent.unwrap());
            }
        } else if left_sibl.is_some() {
            let (left_sibling, curr_left_sib_pos) = left_sibl.unwrap();
            let mut tmp_left_borrow = left_sibling.borrow_mut();

            if let BTreeNode::Leaf {
                parent: l_prnt,
                data,
                next,
            } = &mut *tmp_left_borrow
            {
                let mut tmp_r_borrow = leaf_rc.borrow_mut();

                if let BTreeNode::Leaf {
                    data: curr_node_data,
                    next: curr_next,
                    ..
                } = &mut *tmp_r_borrow
                {
                    match curr_node_data.binary_search_by(|e| e.key.cmp(key)) {
                        Ok(idx) => {
                            curr_node_data.remove(idx);
                        }
                        Err(_) => return false,
                    }

                    data.append(curr_node_data);

                    if curr_next.is_none() {
                        *next = None
                    } else {
                        let new_next = Rc::clone(&curr_next.as_ref().unwrap());
                        *next = Some(new_next);
                    }

                    let t_prnt = l_prnt.as_ref().unwrap().upgrade().unwrap();
                    if let BTreeNode::Internal { keys, children, .. } = &mut *t_prnt.borrow_mut() {
                        keys.remove(curr_left_sib_pos);
                        children.remove(curr_left_sib_pos + 1);

                        if keys.len() < ((self.order + 1) / 2) - 1 {
                            underflowed_parent = Some(Rc::clone(&t_prnt));
                        }
                    }
                    drop(t_prnt);
                }

                drop(tmp_r_borrow);
            }
            drop(tmp_left_borrow);

            if underflowed_parent.is_some() {
                self.fix_parent_underflow(underflowed_parent.unwrap());
            }
        }
        true
    }

    /// Fixes an internal node that has too few keys after a merge
    fn fix_parent_underflow(&mut self, node_rc: Rc<RefCell<BTreeNode<K, V>>>) {
        {
            let node = node_rc.borrow();
            if let BTreeNode::Internal {
                parent,
                keys,
                children,
            } = &*node
            {
                if parent.is_none() {
                    if keys.is_empty() && children.len() == 1 {
                        let child = Rc::clone(&children[0]);
                        match &mut *child.borrow_mut() {
                            BTreeNode::Leaf { parent, .. } | BTreeNode::Internal { parent, .. } => {
                                *parent = None;
                            }
                        }
                        self.root = Some(child);
                    }
                    return;
                }
            }
        }

        let left = BTreeNode::left_sibling(Rc::clone(&node_rc));

        if left.is_none() {
            if let Some((right, _)) = BTreeNode::right_sibling(Rc::clone(&node_rc)) {
                if right.borrow().can_borrow(self.order) {
                    let sep_idx;
                    let parent_rc;

                    {
                        let node = node_rc.borrow();
                        if let BTreeNode::Internal { parent, .. } = &*node {
                            parent_rc = parent.as_ref().unwrap().upgrade().unwrap();
                        } else {
                            return;
                        }
                    }

                    let mut parent = parent_rc.borrow_mut();
                    let (sep_key, moved_child);

                    {
                        let mut right_node = right.borrow_mut();
                        let mut curr_node = node_rc.borrow_mut();

                        if let (
                            BTreeNode::Internal {
                                keys: r_keys,
                                children: r_ch,
                                ..
                            },
                            BTreeNode::Internal {
                                keys: c_keys,
                                children: c_ch,
                                ..
                            },
                            BTreeNode::Internal {
                                keys: p_keys,
                                children: p_ch,
                                ..
                            },
                        ) = (&mut *right_node, &mut *curr_node, &mut *parent)
                        {
                            sep_idx = p_ch.iter().position(|c| Rc::ptr_eq(c, &node_rc)).unwrap();
                            sep_key = p_keys[sep_idx].clone();

                            c_keys.push(sep_key);

                            moved_child = r_ch.remove(0);
                            c_ch.push(Rc::clone(&moved_child));

                            p_keys[sep_idx] = r_keys.remove(0);
                        } else {
                            return;
                        }
                    }

                    match &mut *moved_child.borrow_mut() {
                        BTreeNode::Leaf { parent, .. } | BTreeNode::Internal { parent, .. } => {
                            *parent = Some(Rc::downgrade(&node_rc));
                        }
                    }

                    return;
                }
            }
        }

        if let Some((left_sib, _)) = left {
            if left_sib.borrow().can_borrow(self.order) {
                let sep_idx;
                let parent_rc;

                {
                    let node = node_rc.borrow();
                    if let BTreeNode::Internal { parent, .. } = &*node {
                        parent_rc = parent.as_ref().unwrap().upgrade().unwrap();
                    } else {
                        return;
                    }
                }

                let mut parent = parent_rc.borrow_mut();
                let moved_child;

                {
                    let mut left_node = left_sib.borrow_mut();
                    let mut curr_node = node_rc.borrow_mut();

                    if let (
                        BTreeNode::Internal {
                            keys: l_keys,
                            children: l_ch,
                            ..
                        },
                        BTreeNode::Internal {
                            keys: c_keys,
                            children: c_ch,
                            ..
                        },
                        BTreeNode::Internal {
                            keys: p_keys,
                            children: p_ch,
                            ..
                        },
                    ) = (&mut *left_node, &mut *curr_node, &mut *parent)
                    {
                        sep_idx = p_ch.iter().position(|c| Rc::ptr_eq(c, &node_rc)).unwrap() - 1;

                        c_keys.insert(0, p_keys[sep_idx].clone());

                        moved_child = l_ch.pop().unwrap();
                        c_ch.insert(0, Rc::clone(&moved_child));

                        p_keys[sep_idx] = l_keys.pop().unwrap();
                    } else {
                        return;
                    }
                }

                match &mut *moved_child.borrow_mut() {
                    BTreeNode::Leaf { parent, .. } | BTreeNode::Internal { parent, .. } => {
                        *parent = Some(Rc::downgrade(&node_rc));
                    }
                }

                return;
            }
        }

        let parent_rc = {
            let node = node_rc.borrow();
            if let BTreeNode::Internal { parent, .. } = &*node {
                parent.as_ref().unwrap().upgrade().unwrap()
            } else {
                return;
            }
        };

        let mut parent = parent_rc.borrow_mut();

        let pos = if let BTreeNode::Internal { children, .. } = &*parent {
            children
                .iter()
                .position(|c| Rc::ptr_eq(c, &node_rc))
                .unwrap()
        } else {
            return;
        };

        if let BTreeNode::Internal {
            keys: curr_keys,
            children: curr_children,
            ..
        } = &mut *node_rc.borrow_mut()
        {
            if pos > 0 {
                let left_sibling = if let BTreeNode::Internal { children, .. } = &*parent {
                    Rc::clone(&children[pos - 1])
                } else {
                    return;
                };

                if let BTreeNode::Internal {
                    keys: left_keys,
                    children: left_children,
                    ..
                } = &mut *left_sibling.borrow_mut()
                {
                    if let BTreeNode::Internal {
                        keys: parent_keys,
                        children: parent_children,
                        ..
                    } = &mut *parent
                    {
                        left_keys.push(parent_keys.remove(pos - 1));

                        left_keys.append(curr_keys);
                        left_children.append(curr_children);

                        parent_children.remove(pos);

                        for ch in left_children.iter() {
                            match &mut *ch.borrow_mut() {
                                BTreeNode::Leaf { parent, .. }
                                | BTreeNode::Internal { parent, .. } => {
                                    *parent = Some(Rc::downgrade(&left_sibling));
                                }
                            }
                        }
                    }
                }
            } else {
                let right_sibling = if let BTreeNode::Internal { children, .. } = &*parent {
                    Rc::clone(&children[1])
                } else {
                    return;
                };

                if let BTreeNode::Internal {
                    keys: right_keys,
                    children: right_children,
                    ..
                } = &mut *right_sibling.borrow_mut()
                {
                    if let BTreeNode::Internal {
                        keys: parent_keys,
                        children: parent_children,
                        ..
                    } = &mut *parent
                    {
                        curr_keys.push(parent_keys.remove(0));

                        curr_keys.append(right_keys);
                        curr_children.append(right_children);

                        parent_children.remove(1);

                        for ch in curr_children.iter() {
                            match &mut *ch.borrow_mut() {
                                BTreeNode::Leaf { parent, .. }
                                | BTreeNode::Internal { parent, .. } => {
                                    *parent = Some(Rc::downgrade(&node_rc));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let BTreeNode::Internal { keys, .. } = &*parent {
            if keys.len() < ((self.order + 1) / 2) - 1 {
                drop(parent);
                self.fix_parent_underflow(parent_rc);
            }
        }
    }

    /// Prints the tree level-by-level for debugging
    pub fn _print_tree(&self)
    where
        K: std::fmt::Debug,
    {
        if self.root.is_none() {
            println!("<empty tree>");
            return;
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(Rc::clone(self.root.as_ref().unwrap()));

        let mut level = 0;
        while !queue.is_empty() {
            let level_size = queue.len();
            print!("Level {}: ", level);

            for _ in 0..level_size {
                let node = queue.pop_front().unwrap();
                let n = node.borrow();

                match &*n {
                    BTreeNode::Internal { keys, children, .. } => {
                        print!("(I {:?}) ", keys);

                        for ch in children {
                            queue.push_back(Rc::clone(ch));
                        }
                    }

                    BTreeNode::Leaf { data, .. } => {
                        let keys: Vec<_> = data.iter().map(|e| &e.key).collect();
                        print!("(L [{:?}]) ", keys);
                    }
                }
            }

            println!();
            level += 1;
        }
        println!("===============================");
    }

    /// Returns the leftmost (smallest) leaf node in the tree
    pub fn _leftmost_leaf(&mut self) -> Option<Rc<RefCell<BTreeNode<K, V>>>> {
        if self.root.is_none() {
            return None;
        }
        let root = Rc::clone(self.root.as_ref().unwrap());
        let root_ptr = root.borrow();
        match &*root_ptr {
            BTreeNode::Leaf { .. } => {
                drop(root_ptr);
                return Some(root);
            }
            BTreeNode::Internal { children, .. } => {
                let first_child = Rc::clone(&children[0]);
                drop(root_ptr);
                let mut current = first_child;
                loop {
                    let next = {
                        let node = current.borrow();
                        match &*node {
                            BTreeNode::Leaf { .. } => break,
                            BTreeNode::Internal { children, .. } => Rc::clone(&children[0]),
                        }
                    };
                    current = next;
                }
                return Some(current);
            }
        }
    }
}

impl<K, V> BTreeNode<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn is_leaf(&self) -> bool {
        matches!(self, BTreeNode::Leaf { .. })
    }

    fn is_node_full(&self, order: usize) -> bool {
        match self {
            Self::Leaf { data: keys, .. } => keys.len() > order,
            Self::Internal { keys, .. } => keys.len() > order,
        }
    }

    fn new_internal() -> BTreeNode<K, V> {
        BTreeNode::Internal {
            parent: None,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    fn _new_leaf() -> BTreeNode<K, V> {
        BTreeNode::Leaf {
            parent: None,
            data: Vec::new(),
            next: None,
        }
    }

    pub fn _as_raw_leaf(&mut self) -> Option<LeafNode<'_, K, V>> {
        match self {
            BTreeNode::Leaf { parent, data, next } => Some(LeafNode { parent, data, next }),
            _ => None,
        }
    }

    fn as_mut_leaf<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(
            &mut Option<Weak<RefCell<BTreeNode<K, V>>>>,
            &mut Vec<Entry<K, V>>,
            &mut Option<Rc<RefCell<BTreeNode<K, V>>>>,
        ) -> R,
    {
        match self {
            Self::Leaf { parent, data, next } => {
                let result = f(parent, data, next);
                Some(result)
            }
            _ => None,
        }
    }

    pub fn _as_ref_leaf<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(
            &Option<Weak<RefCell<BTreeNode<K, V>>>>,
            &Vec<Entry<K, V>>,
            &Option<Rc<RefCell<BTreeNode<K, V>>>>,
        ) -> R,
    {
        match self {
            Self::Leaf { parent, data, next } => {
                let res = f(parent, data, next);
                Some(res)
            }
            _ => None,
        }
    }

    pub fn _cmp(&mut self, other: &mut BTreeNode<K, V>, ord: NodeCmpOrd) -> bool {
        let a;
        let b;
        match self._as_raw_leaf() {
            Some(leaf) => {
                a = leaf;
            }
            _ => return false,
        }

        match other._as_raw_leaf() {
            Some(leaf) => {
                b = leaf;
            }
            _ => return false,
        }

        match ord {
            NodeCmpOrd::Less => {
                a.data.last().as_ref().unwrap().key < b.data.first().as_ref().unwrap().key
            }
        }
    }

    pub fn can_borrow(&self, order: usize) -> bool {
        match self {
            BTreeNode::Leaf { data, .. } => {
                (data.len() - 1) as f32 >= (order as f32 / 2.0).ceil() - 1.0
            }
            BTreeNode::Internal { keys, .. } => {
                (keys.len() - 1) as f32 >= (order as f32 / 2.0).ceil() - 1.0
            }
        }
    }

    pub fn left_sibling(
        node_rc: Rc<RefCell<BTreeNode<K, V>>>,
    ) -> Option<(Rc<RefCell<BTreeNode<K, V>>>, usize)> {
        if node_rc.borrow().is_leaf() {
            if let BTreeNode::Leaf { parent, .. } = &*node_rc.borrow() {
                if parent.is_none() {
                    return None;
                }

                let curr_nd_prnt = parent.as_ref().unwrap().upgrade().unwrap();

                if let BTreeNode::Internal {
                    children: prnt_children,
                    ..
                } = &*curr_nd_prnt.borrow()
                {
                    let pos = prnt_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &node_rc));

                    if let Some(0) = pos {
                        return None;
                    }

                    if pos.is_none() {
                        return None;
                    }

                    return Some((
                        Rc::clone(&prnt_children[pos.unwrap() - 1]),
                        pos.unwrap() - 1,
                    ));
                }
            }
        };

        if let BTreeNode::Internal { parent, .. } = &*node_rc.borrow() {
            if parent.as_ref().unwrap().upgrade().is_none() {
                return None;
            }

            let prnt = parent.as_ref().unwrap().upgrade().unwrap();

            if let BTreeNode::Internal {
                children: prnt_ch, ..
            } = &*prnt.borrow()
            {
                let pos = prnt_ch
                    .iter()
                    .position(|child| Rc::ptr_eq(child, &node_rc))
                    .unwrap();

                if pos == 0 {
                    return None;
                }

                return Some((Rc::clone(&prnt_ch[pos - 1]), pos - 1));
            }
        }
        None
    }

    pub fn right_sibling(
        node_rc: Rc<RefCell<BTreeNode<K, V>>>,
    ) -> Option<(Rc<RefCell<BTreeNode<K, V>>>, usize)> {
        if let BTreeNode::Leaf { parent, .. } = &*node_rc.borrow() {
            let parent = parent.as_ref()?.upgrade()?;

            if let BTreeNode::Internal { children, .. } = &*parent.borrow() {
                let pos = children
                    .iter()
                    .position(|child| Rc::ptr_eq(child, &node_rc))?;

                if pos + 1 < children.len() {
                    return Some((Rc::clone(&children[pos + 1]), pos + 1));
                } else {
                    return None;
                }
            }
        }

        if let BTreeNode::Internal { parent, .. } = &*node_rc.borrow() {
            let parent = parent.as_ref()?.upgrade()?;

            if let BTreeNode::Internal { children, .. } = &*parent.borrow() {
                let pos = children
                    .iter()
                    .position(|child| Rc::ptr_eq(child, &node_rc))?;

                if pos + 1 < children.len() {
                    return Some((Rc::clone(&children[pos + 1]), pos + 1));
                } else {
                    return None;
                }
            }
        }

        None
    }
}

impl<K, V> Iterator for BTreeNode<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    type Item = Rc<RefCell<BTreeNode<K, V>>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Leaf { next, .. } => match next {
                Some(c_next) => {
                    let next = Rc::clone(c_next);
                    Some(next)
                }
                None => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use std::collections::HashSet;

    fn e(k: i32) -> Entry<i32, String> {
        Entry {
            key: k,
            val: k.to_string(),
        }
    }

    #[test]
    fn test_sorted_insert() {
        let mut t = BTree::new(128);

        for i in 0..100_000 {
            t.insert(Entry {
                key: i,
                val: format!("{}", i),
            });
        }

        for i in 0..100_000 {
            let res = t.search(&i);
            assert!(res.is_some(), "Missing key {}", i);
        }
    }

    #[test]
    fn test_random_insert() {
        let mut t = BTree::new(4);

        let mut v: Vec<_> = (0..100_000).collect();
        v.shuffle(&mut rand::rng());

        for i in &v {
            t.insert(Entry {
                key: *i,
                val: format!("{}", i),
            });
        }

        for i in 0..10_000 {
            assert!(t.search(&i).is_some(), "Missing key {}", i);
        }
    }

    #[test]
    fn test_linked_leaves() {
        let mut t = BTree::new(64);
        for i in 0..1000 {
            t.insert(Entry {
                key: i,
                val: format!("{}", i),
            });
        }

        let leftmost_leaf = t._leftmost_leaf();
        match leftmost_leaf {
            Some(mut current_node) => loop {
                let next = {
                    let curr = &mut *current_node.borrow_mut();
                    if let Some(node) = curr.next() {
                        if curr._cmp(&mut *node.borrow_mut(), NodeCmpOrd::Less) {
                            node
                        } else {
                            panic!();
                        }
                    } else {
                        break;
                    }
                };

                current_node = next;
            },
            None => unreachable!(),
        }
    }

    #[test]
    fn delete_simple_no_underflow() {
        let mut t = BTree::new(5);

        for k in [10, 20, 30, 40, 50] {
            t.insert(e(k));
        }

        t.delete(&10);

        for k in [20, 30, 40, 50] {
            assert!(t.search(&k).is_some(), "Key {} missing", k);
        }

        assert!(t.search(&10).is_none());
    }

    #[test]
    fn test_generic_string_keys() {
        let mut t = BTree::new(5);

        for s in ["apple", "banana", "cherry", "date", "elderberry"] {
            t.insert(Entry {
                key: s.to_string(),
                val: s.len(),
            });
        }

        assert!(t.search(&"banana".to_string()).is_some());
        assert!(t.search(&"grape".to_string()).is_none());

        t.delete(&"cherry".to_string());
        assert!(t.search(&"cherry".to_string()).is_none());
    }

    #[test]
    fn test_update_existing_key() {
        let mut t = BTree::new(5);

        t.insert(Entry {
            key: 42,
            val: "first".to_string(),
        });

        let result = t.search(&42);
        assert_eq!(result.unwrap().val, "first");

        t.insert(Entry {
            key: 42,
            val: "updated".to_string(),
        });

        let result = t.search(&42);
        assert_eq!(result.unwrap().val, "updated");
    }
}

#[cfg(test)]
mod complex_tests {
    use super::*;
    use rand::seq::SliceRandom;
    use std::collections::HashSet;

    fn e(k: i32) -> Entry<i32, String> {
        Entry {
            key: k,
            val: k.to_string(),
        }
    }

    #[test]
    fn test_complex_mixed_operations() {
        let mut t = BTree::new(5);
        let n = 1000;

        // Phase 1: Insert sequential keys
        for i in 0..n {
            t.insert(Entry {
                key: i,
                val: format!("value_{}", i),
            });
        }

        // Verify all inserted
        for i in 0..n {
            assert!(t.search(&i).is_some(), "Key {} missing after insert", i);
        }

        // Phase 2: Delete every third key
        for i in (0..n).step_by(3) {
            assert!(t.delete(&i), "Failed to delete key {}", i);
            assert!(
                t.search(&i).is_none(),
                "Key {} still exists after delete",
                i
            );
        }

        // Phase 3: Insert new keys in gaps
        for i in (0..n).step_by(3) {
            t.insert(Entry {
                key: i + n,
                val: format!("new_value_{}", i + n),
            });
        }

        // Phase 4: Verify correct state
        for i in 0..n {
            if i % 3 == 0 {
                assert!(t.search(&i).is_none(), "Deleted key {} exists", i);
                assert!(t.search(&(i + n)).is_some(), "New key {} missing", i + n);
            } else {
                assert!(t.search(&i).is_some(), "Key {} missing", i);
            }
        }
    }

    #[test]
    fn test_stress_alternating_insert_delete() {
        let mut t = BTree::new(7);
        let mut present = HashSet::new();

        // Complex pattern: insert 5, delete 2, repeat
        for cycle in 0..200 {
            let base = cycle * 5;

            // Insert 5 keys
            for offset in 0..5 {
                let key = base + offset;
                t.insert(e(key));
                present.insert(key);
            }

            // Delete 2 random keys from present set
            if present.len() >= 2 {
                let to_delete: Vec<_> = present.iter().take(2).copied().collect();
                for key in to_delete {
                    assert!(t.delete(&key), "Failed to delete key {}", key);
                    present.remove(&key);
                }
            }

            // Verify all present keys exist
            for &key in &present {
                assert!(
                    t.search(&key).is_some(),
                    "Key {} missing at cycle {}",
                    key,
                    cycle
                );
            }
        }

        // Final verification
        for &key in &present {
            assert!(
                t.search(&key).is_some(),
                "Key {} missing in final check",
                key
            );
        }
    }

    #[test]
    fn test_cascade_merges_with_height_reduction() {
        let mut t = BTree::new(3);

        // Build a tree with multiple levels
        for i in 0..100 {
            t.insert(e(i));
        }

        // Verify tree has data
        for i in 0..100 {
            assert!(t.search(&i).is_some());
        }

        // Delete most keys to trigger cascading merges
        for i in 0..90 {
            t.delete(&i);
        }

        // Verify remaining keys
        for i in 90..100 {
            assert!(
                t.search(&i).is_some(),
                "Key {} missing after cascade delete",
                i
            );
        }

        // Verify deleted keys are gone
        for i in 0..90 {
            assert!(t.search(&i).is_none(), "Key {} still exists", i);
        }
    }

    #[test]
    fn test_random_operations_with_verification() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut t = BTree::new(5);
        let mut expected = HashSet::new();

        for _ in 0..5000 {
            let op = rng.gen_range(0..3);
            let key = rng.gen_range(0..500);

            match op {
                0 => {
                    // Insert
                    t.insert(e(key));
                    expected.insert(key);
                }
                1 => {
                    // Delete
                    let deleted = t.delete(&key);
                    let was_present = expected.remove(&key);
                    assert_eq!(deleted, was_present, "Delete inconsistency for key {}", key);
                }
                2 => {
                    // Search
                    let found = t.search(&key).is_some();
                    let should_exist = expected.contains(&key);
                    assert_eq!(found, should_exist, "Search inconsistency for key {}", key);
                }
                _ => unreachable!(),
            }
        }

        // Final consistency check
        for key in 0..500 {
            let found = t.search(&key).is_some();
            let should_exist = expected.contains(&key);
            assert_eq!(found, should_exist, "Final check failed for key {}", key);
        }
    }

    #[test]
    fn test_boundary_conditions_min_order() {
        let mut t = BTree::new(3); // Minimum practical order

        // Insert until we force multiple splits
        for i in 0..50 {
            t.insert(e(i));
        }

        // Verify all keys
        for i in 0..50 {
            assert!(t.search(&i).is_some());
        }

        // Delete in reverse to test different merge patterns
        for i in (0..50).rev() {
            t.delete(&i);
            assert!(t.search(&i).is_none());

            // Verify remaining keys still exist
            for j in 0..i {
                assert!(
                    t.search(&j).is_some(),
                    "Key {} missing after deleting {}",
                    j,
                    i
                );
            }
        }
    }

    #[test]
    fn test_duplicate_key_updates() {
        let mut t = BTree::new(5);

        // Insert initial keys
        for i in 0..100 {
            t.insert(Entry {
                key: i,
                val: format!("initial_{}", i),
            });
        }

        // Update all keys with new values
        for i in 0..100 {
            t.insert(Entry {
                key: i,
                val: format!("updated_{}", i),
            });
        }

        // Verify updates took effect
        for i in 0..100 {
            let entry = t.search(&i).unwrap();
            assert_eq!(
                entry.val,
                format!("updated_{}", i),
                "Key {} not updated correctly",
                i
            );
        }

        // Update random subset again
        for i in (0..100).step_by(7) {
            t.insert(Entry {
                key: i,
                val: format!("final_{}", i),
            });
        }

        // Verify mixed state
        for i in 0..100 {
            let entry = t.search(&i).unwrap();
            let expected = if i % 7 == 0 {
                format!("final_{}", i)
            } else {
                format!("updated_{}", i)
            };
            assert_eq!(entry.val, expected, "Key {} has wrong value", i);
        }
    }

    #[test]
    fn test_interleaved_insert_delete_patterns() {
        let mut t = BTree::new(7);

        // Pattern 1: Insert even numbers
        for i in (0..200).step_by(2) {
            t.insert(e(i));
        }

        // Pattern 2: Insert odd numbers
        for i in (1..200).step_by(2) {
            t.insert(e(i));
        }

        // Verify all present
        for i in 0..200 {
            assert!(t.search(&i).is_some());
        }

        // Pattern 3: Delete multiples of 3
        for i in (0..200).step_by(3) {
            t.delete(&i);
        }

        // Pattern 4: Delete multiples of 5 (that aren't already deleted)
        for i in (0..200).step_by(5) {
            t.delete(&i);
        }

        // Verify correct keys remain
        for i in 0..200 {
            let should_exist = i % 3 != 0 && i % 5 != 0;
            let exists = t.search(&i).is_some();
            assert_eq!(exists, should_exist, "Key {} existence mismatch", i);
        }
    }

    #[test]
    fn test_large_scale_random_workload() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut t = BTree::new(10);
        let mut keys_inserted = HashSet::new();

        // Phase 1: Bulk random insert
        let mut insert_keys: Vec<i32> = (0..10000).collect();
        insert_keys.shuffle(&mut rand::thread_rng());

        for key in insert_keys.iter().take(5000) {
            t.insert(e(*key));
            keys_inserted.insert(*key);
        }

        // Phase 2: Random deletes (30% of inserted)
        let to_delete: Vec<_> = keys_inserted.iter().take(1500).copied().collect();

        for key in to_delete {
            assert!(t.delete(&key));
            keys_inserted.remove(&key);
        }

        // Phase 3: More random inserts
        for key in insert_keys.iter().skip(5000).take(3000) {
            t.insert(e(*key));
            keys_inserted.insert(*key);
        }

        // Phase 4: Random verification samples
        for _ in 0..1000 {
            let key = rng.gen_range(0..10000);
            let found = t.search(&key).is_some();
            let should_exist = keys_inserted.contains(&key);
            assert_eq!(found, should_exist, "Verification failed for key {}", key);
        }

        // Phase 5: Complete verification
        for key in 0..10000 {
            let found = t.search(&key).is_some();
            let should_exist = keys_inserted.contains(&key);
            assert_eq!(
                found, should_exist,
                "Final verification failed for key {}",
                key
            );
        }
    }

    #[test]
    fn test_sequential_delete_causing_multiple_borrows() {
        let mut t = BTree::new(4);

        // Create specific structure
        for i in 0..60 {
            t.insert(e(i));
        }

        // Delete keys in pattern that forces borrows
        for i in (10..30).step_by(2) {
            t.delete(&i);
        }

        // Verify structure integrity
        for i in 0..60 {
            let should_exist = !(i >= 10 && i < 30 && i % 2 == 0);
            let exists = t.search(&i).is_some();
            assert_eq!(exists, should_exist, "Key {} state wrong", i);
        }
    }

    #[test]
    fn test_string_keys_complex() {
        let mut t = BTree::new(5);
        let words = vec![
            "apple",
            "banana",
            "cherry",
            "date",
            "elderberry",
            "fig",
            "grape",
            "honeydew",
            "kiwi",
            "lemon",
            "mango",
            "nectarine",
            "orange",
            "papaya",
            "quince",
            "raspberry",
            "strawberry",
            "tangerine",
            "ugli",
            "vanilla",
            "watermelon",
            "xigua",
            "yuzu",
            "zucchini",
        ];

        // Insert all words
        for word in &words {
            t.insert(Entry {
                key: word.to_string(),
                val: word.len(),
            });
        }

        // Verify all present
        for word in &words {
            let result = t.search(&word.to_string());
            assert!(result.is_some(), "Word {} not found", word);
            assert_eq!(result.unwrap().val, word.len());
        }

        // Delete words with odd lengths
        for word in &words {
            if word.len() % 2 == 1 {
                t.delete(&word.to_string());
            }
        }

        // Verify correct state
        for word in &words {
            let exists = t.search(&word.to_string()).is_some();
            let should_exist = word.len() % 2 == 0;
            assert_eq!(exists, should_exist, "Word {} existence mismatch", word);
        }
    }

    #[test]
    fn test_extreme_delete_until_empty() {
        let mut t = BTree::new(5);
        let n = 500;

        // Fill tree
        for i in 0..n {
            t.insert(e(i));
        }

        // Delete all in random order
        let mut keys: Vec<_> = (0..n).collect();
        keys.shuffle(&mut rand::thread_rng());

        for (idx, key) in keys.iter().enumerate() {
            assert!(
                t.delete(key),
                "Failed to delete key {} at step {}",
                key,
                idx
            );

            // Verify this key is gone
            assert!(
                t.search(key).is_none(),
                "Key {} still exists after delete",
                key
            );

            // Sample check: verify some remaining keys still exist
            if idx % 50 == 0 && idx < keys.len() - 1 {
                for check_key in keys.iter().skip(idx + 1).take(10) {
                    assert!(
                        t.search(check_key).is_some(),
                        "Key {} disappeared prematurely at step {}",
                        check_key,
                        idx
                    );
                }
            }
        }

        // Tree should be empty
        for i in 0..n {
            assert!(
                t.search(&i).is_none(),
                "Tree not empty: key {} still exists",
                i
            );
        }
    }

    #[test]
    fn test_repeated_insert_delete_same_keys() {
        let mut t = BTree::new(5);

        // Repeat cycle: insert -> delete -> insert -> delete
        for cycle in 0..100 {
            for key in 0..50 {
                t.insert(e(key));
            }

            for key in 0..50 {
                assert!(
                    t.search(&key).is_some(),
                    "Key {} missing after insert in cycle {}",
                    key,
                    cycle
                );
            }

            for key in 0..50 {
                assert!(
                    t.delete(&key),
                    "Failed to delete key {} in cycle {}",
                    key,
                    cycle
                );
            }

            for key in 0..50 {
                assert!(
                    t.search(&key).is_none(),
                    "Key {} still exists after delete in cycle {}",
                    key,
                    cycle
                );
            }
        }
    }

    #[test]
    fn test_split_merge_balance() {
        let mut t = BTree::new(4);

        // Insert to cause splits
        for i in 0..100 {
            t.insert(e(i));
        }

        // Delete to cause merges
        for i in 0..50 {
            t.delete(&i);
        }

        // Re-insert to cause splits again
        for i in 0..50 {
            t.insert(e(i + 100));
        }

        // Verify all correct keys exist
        for i in 50..100 {
            assert!(t.search(&i).is_some(), "Original key {} missing", i);
        }
        for i in 100..150 {
            assert!(t.search(&i).is_some(), "New key {} missing", i);
        }
        for i in 0..50 {
            assert!(t.search(&i).is_none(), "Deleted key {} exists", i);
        }
    }

    #[test]
    fn test_custom_type_keys() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Person {
            age: u32,
            name: String,
        }

        let mut t = BTree::new(5);

        let people = vec![
            Person {
                age: 25,
                name: "Alice".to_string(),
            },
            Person {
                age: 30,
                name: "Bob".to_string(),
            },
            Person {
                age: 25,
                name: "Charlie".to_string(),
            }, // Same age, different name
            Person {
                age: 35,
                name: "Diana".to_string(),
            },
            Person {
                age: 30,
                name: "Eve".to_string(),
            },
        ];

        // Insert all
        for person in &people {
            t.insert(Entry {
                key: person.clone(),
                val: format!("Employee: {}", person.name),
            });
        }

        // Search for specific people
        for person in &people {
            let result = t.search(person);
            assert!(result.is_some(), "Person {:?} not found", person);
        }

        // Delete someone
        t.delete(&people[1]);
        assert!(t.search(&people[1]).is_none());

        // Others still exist
        assert!(t.search(&people[0]).is_some());
        assert!(t.search(&people[2]).is_some());
    }

    #[test]
    fn test_high_order_tree_performance() {
        let mut t = BTree::new(100); // High order = fewer splits

        // Insert many keys
        for i in 0..50000 {
            t.insert(e(i));
        }

        // Random access pattern
        for i in (0..50000).step_by(7) {
            assert!(t.search(&i).is_some());
        }

        // Delete pattern
        for i in (0..50000).step_by(11) {
            t.delete(&i);
        }

        // Verify correct state
        for i in 0..50000 {
            let should_exist = i % 11 != 0;
            let exists = t.search(&i).is_some();
            assert_eq!(exists, should_exist, "Key {} state wrong", i);
        }
    }

    #[test]
    fn test_edge_case_single_key_operations() {
        let mut t = BTree::new(5);

        // Single insert
        t.insert(e(42));
        assert!(t.search(&42).is_some());

        // Single delete
        assert!(t.delete(&42));
        assert!(t.search(&42).is_none());

        // Insert again
        t.insert(e(42));
        assert!(t.search(&42).is_some());

        // Try deleting non-existent
        assert!(!t.delete(&999));

        // Key 42 should still exist
        assert!(t.search(&42).is_some());
    }

    #[test]
    fn test_linked_list_integrity_after_operations() {
        let mut t = BTree::new(5);

        // Insert keys
        for i in 0..100 {
            t.insert(e(i));
        }

        // Delete some keys
        for i in (20..80).step_by(3) {
            t.delete(&i);
        }

        // Walk the linked list
        if let Some(mut current) = t._leftmost_leaf() {
            let mut prev_max = -1;
            loop {
                let (min_key, has_next) = {
                    let node = current.borrow();
                    if let BTreeNode::Leaf { data, next, .. } = &*node {
                        let min = data.first().map(|e| e.key).unwrap_or(i32::MAX);
                        (min, next.is_some())
                    } else {
                        panic!("Expected leaf node");
                    }
                };

                // Keys should be increasing across leaves
                assert!(
                    min_key > prev_max,
                    "Linked list order violated: {} not > {}",
                    min_key,
                    prev_max
                );
                prev_max = min_key;

                if !has_next {
                    break;
                }

                current = {
                    let mut node = current.borrow_mut();
                    node.next().unwrap()
                };
            }
        }
    }
}
