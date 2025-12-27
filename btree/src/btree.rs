//
#![allow(unused)]
//
//
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

        // PHASE 1: Navigate down the tree to find the correct leaf
        // This loop continues until we reach a leaf node.
        // At each internal node, we decide which child to follow based on key comparisons.
        let _ = loop {
            let next = {
                let node = current.borrow();
                match &*node {
                    BTreeNode::Internal { keys, children, .. } => {
                        // INTERNAL NODE NAVIGATION
                        // Internal nodes act as signposts. Their keys tell us which path to take.
                        // 
                        // Example with keys [20, 40, 60]:
                        //
                        //         [20 | 40 | 60]
                        //        /    |    |    \
                        //     child0 ch1 ch2   ch3
                        //     (<20) (20-40) (40-60) (>=60)
                        //
                        // If searching for 35:
                        //   - 35 >= 20, so skip child0
                        //   - 35 < 40, so take child1
                        
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
        //
        //
        // This traverses the tree from root to leaf, following the path
        // that the key would take during a search operation
        let leaf = self.find_leaf(&entry.key);

        // Insert the entry into the leaf and maintain sorted order
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
        // Check if we need to split due to overflow
        let is_leaf = (*leaf.borrow()).is_leaf();

        if (*leaf.borrow()).is_node_full(self.order) {
            if is_leaf {
                // LEAF SPLIT PROCESS:
                // 1. Create a new right sibling leaf
                // 2. Move half the entries to the new leaf
                // 3. Update the linked list pointers (next)
                // 4. Promote the middle key to the parent
                // 5. If parent overflows, split it recursively
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
                        // Same as in the search function: compare key against internal keys
                        // to determine which child subtree to follow
                        //
                        // Visual example with keys [30, 60]:
                        //
                        //         [30 | 60]
                        //        /    |    \
                        //     (<30) (30-60) (>=60)
                        //
                        // For key=45: 45>=30, 45<60, so take middle child
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
        // No parent exists (we're at the root)
        if parent.is_none() {
            // Create a new internal node to become the parent
            let parent = Rc::new(RefCell::new(BTreeNode::new_internal()));
            // Since there's no parent, this node must become the new root
            // This is how the tree grows in height
            self.root = Some(parent);
            return self.root.as_ref().map(Rc::downgrade).unwrap();
        }
        // Upgrade the weak reference to a strong one, then downgrade again
        // This dance is necessary because we need to return a Weak reference
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
        //
        // BORROW SCOPE BEGINS: We have mutable access to the overflowing leaf
        // ^^^^^^^^^^^^^^^^^^^

        if let BTreeNode::Leaf {
            parent: left_child_parent,
            data: left_keys,
            next,
        } = &mut *mut_leaf_ptr
        {
            // The parent will receive:
            //   - A pointer to the left child (original leaf)
            //   - The separator key
            //   - A pointer to the right child (new leaf we're about to create)
            let ptr_leaf_parent = (self.get_node_parent(left_child_parent).upgrade()).unwrap();
            let mut leaf_parent = ptr_leaf_parent.borrow_mut();

            if let BTreeNode::Internal {
                keys: parent_keys,
                children: parent_children,
                ..
            } = &mut *leaf_parent
            {
                // For a leaf with 4 entries [10, 20, 30, 40]:
                //   - middle = 4 / 2 = 2
                //   - key_to_promote = left_keys[2].key = 30
                //   - Left keeps: [10, 20]
                //   - Right gets: [30, 40]
                let middle = left_keys.len() / 2;
                let key_to_promote = left_keys[middle].key.clone();

                let right_child = {
                    let tmp = Rc::new(RefCell::new(BTreeNode::Leaf {
                        // Point back to the parent we just obtained
                        parent: Some(Rc::downgrade(&ptr_leaf_parent)),
                        // split_off(middle) removes elements [middle..] from left_keys
                        // and returns them as a new Vec
                        // After this, left_keys contains [0..middle]
                        data: left_keys.split_off(middle),
                        // The right child should point to whatever the original leaf
                        // was pointing to (could be None or another leaf)
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
                // Update pointers to maintain tree structure

                // Left child (original leaf) now points to the parent
                *left_child_parent = Some(Rc::downgrade(&ptr_leaf_parent));

                // Left child's 'next' pointer now points to right sibling
                // This maintains the linked list of leaves for range scans
                *next = Some(Rc::clone(&right_child));
                

                // SPECIAL CASE: Root split

                // This is the first split ever. The parent is empty and just became root.
                    // Tree structure before:  [10, 20, 30, 40] (single leaf, is root)
                    // Tree structure after:
                    //                              [30]
                    //                             /    \
                    //                        [10,20]  [30,40]

                if parent_children.is_empty() {
                    parent_children.push(Rc::clone(&leaf));
                    parent_keys.push(key_to_promote);
                    parent_children.push(Rc::clone(&right_child));
                } else {
                    // NORMAL CASE: Parent already has children
                    // Find where the left (original) leaf is in parent's children array
                    

                    let left_index = parent_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &leaf))
                        .unwrap();

                    
                    // Insert the separator key at the appropriate position
                    // Example: if left_index=1, parent keys [20, 40, 60]
                    //          inserting 30 at position 1 gives [20, 30, 40, 60]

                    parent_keys.insert(left_index, key_to_promote);
                    parent_children.insert(left_index + 1, Rc::clone(&right_child));
                }

                if parent_keys.len() >= self.order {
                    // Parent now has too many keys and needs to be split
                    // We'll handle this after we release all our borrows
                    overflowed_parent = Some(Rc::clone(&ptr_leaf_parent));
                }
            }
            // Manually drop borrows to prevent borrowing conflicts later
            // This is necessary because we might need to recursively split the parent 
            drop(leaf_parent);
            drop(mut_leaf_ptr);
        }

        if overflowed_parent.is_some() {
            // If the parent overflowed, we need to split it too
            // This can cascade all the way up to the root
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
        // We're splitting an internal node that looks like this:
        //
        //              PARENT
        //                 |
        //                 v
        //         LEFT NODE (to split)
        //        /  |  |  \   (has too many children)
        //       /   |  |   \
        //      C0  C1 C2  C3
        //
        // After split:
        //
        //              PARENT
        //             /      \
        //       LEFT NODE  RIGHT NODE
        //        /    \      /    \
        //       C0    C1    C2    C3
        //
        // The middle key from LEFT NODE gets promoted to PARENT
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode<K, V>>>> = None;

        let mut mut_left_ptr = node.borrow_mut();
        if let BTreeNode::Internal {
            parent: left_node_parent,
            keys: left_node_keys,
            children: left_node_children,
        } = &mut *mut_left_ptr
        {
            // Get or create parent
            let parent = self.get_node_parent(left_node_parent).upgrade().unwrap();
            let mut mut_parent_ptr = parent.borrow_mut();
            // For keys [3, 5, 7, 9] with order=4:
            //   - middle = 4 / 2 = 2
            //   - key_to_promote = left_node_keys[2] = 7
            //   - After split:
            //       Left gets:  [3, 5]
            //       Right gets: [9]
            //       Parent gets: 7 (the separator)
            let middle = left_node_keys.len() / 2;
            let key_to_promote = left_node_keys[middle].clone();

            let right_node = Rc::new(RefCell::new(BTreeNode::Internal {
                parent: Some(Rc::downgrade(&parent)),
                keys: {
                    // split_off(middle + 1) gives us keys [middle+1..]
                    // Example: [3,5,7,9] -> split_off(3) -> [9]
                    let keys = left_node_keys.split_off(middle + 1);
                    // IMPORTANT: For internal nodes, the middle key goes UP to parent
                    // It should NOT stay in the left node
                    // So we pop it from left_node_keys
                    left_node_keys.pop();
                    keys
                },
                // Split children similarly: right gets children [middle+1..]
                children: left_node_children.split_off(middle + 1),
            }));

            // Update left node's parent pointer
            *left_node_parent = Some(Rc::downgrade(&parent));

            // All children in left_node_children need to point back to 'node'
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
            // Fix parent pointers for right node's children
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
            // Release borrow early to prevent conflicts
            drop(mut_right_ptr);

            // Insert split results into parent
            if let BTreeNode::Internal {
                keys: parent_keys,
                children: parent_children,
                ..
            } = &mut *mut_parent_ptr
            {
                if parent_children.is_empty() {
                    // ROOT SPLIT CASE
                    //
                    // The parent is empty, meaning it's a brand new root
                    // This increases the tree height
                    parent_children.push(Rc::clone(&node));
                    parent_keys.push(key_to_promote);
                    parent_children.push(Rc::clone(&right_node));
                } else {
                    // NORMAL CASE
                    
                    // Find where the left node is in parent's children
                    let left_index = parent_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &node))
                        .unwrap();
                    // Insert separator key
                    parent_keys.insert(left_index, key_to_promote);

                    // Insert right child after left child
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
    ///
    /// Returns true if key was found and deleted, false otherwise
    ///
    /// The process handles several cases:
    /// 1. Simple delete (node still has enough keys)
    /// 2. Borrow from right sibling
    /// 3. Borrow from left sibling  
    /// 4. Merge with sibling
    ///
    /// Example of borrowing:
    /// ```text
    /// Before delete(10):
    ///        [30]
    ///       /    \
    ///   [10,20]  [30,40,50]
    ///
    /// After (borrowed 30 from right):
    ///        [40]
    ///       /    \
    ///    [20,30]  [40,50]
    /// ```
    pub fn delete(&mut self, key: &K) -> bool {
        // OVERVIEW OF DELETION

        // B+Tree deletion is more complex than insertion because we need to
        // maintain the minimum number of keys in each node.
        //
        // Minimum keys per node = ceil(order/2) - 1
        // For order=5: minimum = 2 keys per node
        //
        // When a deletion causes underflow, we have options:
        //   1. Borrow from a sibling (if it has spare keys)
        //   2. Merge with a sibling (if borrowing isn't possible)
        //   3. Merges can cascade up the tree

        // Find the leaf containing the key
        let leaf_rc = self.find_leaf(key);


        // Special case - deleting from root
        if Rc::ptr_eq(&leaf_rc, self.root.as_ref().unwrap()) {
            let state = leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                match data.binary_search_by(|entry| entry.key.cmp(key)) {
                    Ok(idx) => {
                        //  Remove the key directly - no underflow concerns for root
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

        // This analyzes the node and its siblings to determine:
        //   - Can we delete without underflow? (Simple)
        //   - Can we borrow from right sibling? (RightBorrow)
        //   - Can we borrow from left sibling? (LeftBorrow)
        //   - Must we merge? (Merge)

        let plan = self.delete_planner(key, Rc::clone(&leaf_rc));


        // Execute the deletion plan
        match plan {
            (DeletePlanner::Empty, ..) => {
                return false;
            }

            (DeletePlanner::Simple, ..) => {
                // Node will still have enough keys after deletion
                // Just remove the key directly

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
                // BORROW FROM RIGHT
                
                // Right sibling has extra keys
                // Move one key from right sibling to current node
                return self.right_borrow(Rc::clone(&leaf_rc), key, r_leaf, pos);
            }

            (DeletePlanner::LeftBorrow, left_sibl, left_sibl_pos) => {
                // BORROW FROM LEFT
                
                // Left sibling has extra keys
                // Move one key from left sibling to current node
                return self.left_borrow(key, leaf_rc, left_sibl, left_sibl_pos);
            }

            _ => {
                // MERGE CASE
                
                // Neither sibling can lend a key
                // Must merge this node with a sibling
                return self.merge_leaf(leaf_rc, key);
            }
        }

    }

    /// Decides which deletion strategy to use based on the tree state
    /// Returns the plan along with sibling info if needed
    fn delete_planner(
        &mut self,
        _: &K,
        leaf_rc: Rc<RefCell<BTreeNode<K, V>>>,
    ) -> (DeletePlanner, Rc<RefCell<BTreeNode<K, V>>>, usize) {

        // DELETION PLANNING ALGORITHM
        

        // This function determines the safest way to delete a key
        // without violating B+Tree properties
        //
        // Priority order:
        //   1. Check if tree is empty
        //   2. Check if simple deletion works (node stays valid)
        //   3. Try borrowing from right sibling
        //   4. Try borrowing from left sibling
        //   5. Fall back to merge


        // Is this an empty root?
        if let BTreeNode::Internal { parent, keys, .. } = &*leaf_rc.borrow() {
            if parent.is_none() {
                if keys.is_empty() {
                    return (DeletePlanner::Empty, Rc::clone(&leaf_rc), 0x00);
                }
            }
        }

        // Simple delete works if: (keys_after_delete) >= ceil(order/2) - 1
        //
        // Example with order=5:
        //   Minimum keys = ceil(5/2) - 1 = 2
        //   If node has 3 keys, deleting 1 leaves 2 (OK!)
        //   If node has 2 keys, deleting 1 leaves 1 (UNDERFLOW!)

        let tmp_borrow = leaf_rc.borrow();
        if let BTreeNode::Leaf { parent, data, .. } = &*tmp_borrow {
            if parent.is_none() {
                if data.is_empty() {
                    return (DeletePlanner::Empty, Rc::clone(&leaf_rc), 0x00);
                }
            }
        }
        drop(tmp_borrow);

        // Borrowing works if the sibling has more than the minimum keys
        //
        // Example with order=5 (min=2 keys):
        //   Current:  [10]      (will underflow after delete)
        //   Right:    [30, 40, 50]  (has 3 > 2, can lend one!)
        //
        // After borrowing:
        //   Current:  [30]
        //   Right:    [40, 50]
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

        // Similar to right borrow, but takes from left sibling instead
        //
        // Example:
        //   Left:     [10, 20, 30]  (has spare keys)
        //   Current:  [40]          (will underflow)
        //
        // After borrowing:
        //   Left:     [10, 20]
        //   Current:  [30, 40]

        let left_sibl = BTreeNode::left_sibling(Rc::clone(&leaf_rc));
        if left_sibl.is_some() {
            let (left, pos) = left_sibl.unwrap();
            if left.borrow().can_borrow(self.order) {
                return (DeletePlanner::LeftBorrow, left, pos);
            }
        }

        // FALLBACK: Must merge
        // If we can't borrow from either sibling, we must merge
        (DeletePlanner::Merge, Rc::clone(&leaf_rc), 0x03)
    }

    /// Handles delete by borrowing a key from the right sibling
    /// Also updates the parent separator key
    pub fn right_borrow(
        &mut self,
        leaf: Rc<RefCell<BTreeNode<K, V>>>,
        deleted_key: &K,
        right_sibl: Rc<RefCell<BTreeNode<K, V>>>,
        right_sibling_pos: usize,
    ) -> bool {
        // RIGHT BORROW PROCESS
        // 
        // Before:
        //           Parent: [40]
        //           /           \
        //     Current:[10,20]  Right:[40,50,60]
        //
        // Delete 10, borrow from right:
        //           Parent: [50]  <- separator updated
        //           /           \
        //     Current:[20,40]  Right:[50,60]

        if let BTreeNode::Leaf { parent, data, .. } = &mut *leaf.borrow_mut() {
            // Delete the target key
            match data.binary_search_by(|e| e.key.cmp(deleted_key)) {
                Ok(idx) => {
                    data.remove(idx);
                }
                Err(_) => return false,
            }

            // Borrow first entry from right sibling
            if let BTreeNode::Leaf { data: sib_dat, .. } = &mut *right_sibl.borrow_mut() {
                let entry = sib_dat.remove(0);

                // Binary search to find insertion position
                match data.binary_search_by(|e| e.key.cmp(&entry.key)) {
                    Ok(idx) | Err(idx) => data.insert(idx, entry),
                }

                // Update parent separator key

                // The separator key between current and right sibling
                // must now be the first key of the right sibling
                let new_sep = sib_dat[0].key.clone(); // MAY not implement the copy trait, just CLONE.

                let prnt = parent.as_ref().unwrap().upgrade().unwrap();
                if let BTreeNode::Internal { keys, .. } = &mut *prnt.borrow_mut() {
                    // Update the separator at position right_sibling_pos - 1
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
        // LEFT BORROW PROCESS
        
        //
        // Before:
        //           Parent: [40]
        //           /           \
        //     Left:[10,20,30]  Current:[40,50]
        //
        // Delete 50, borrow from left:
        //           Parent: [30]  <- separator updated
        //           /           \
        //     Left:[10,20]     Current:[30,40]

        if let BTreeNode::Leaf { parent, data, .. } = &mut *leaf_rc.borrow_mut() {
            // Delete the target key
            match data.binary_search_by(|e| e.key.cmp(key)) {
                Ok(idx) => {
                    data.remove(idx);
                }
                Err(_) => return false,
            }
            // Borrow last entry from left sibling
            if let BTreeNode::Leaf { data: left_dat, .. } = &mut *left_sibl.borrow_mut() {
                let last_idx = left_dat.len() - 1;
                let borrowed_entry = left_dat.remove(last_idx);
                let t_key = borrowed_entry.key.clone();
                // Remove last from left, insert at beginning of current
                data.insert(0, borrowed_entry);

                // The separator between left and current should now be
                // the key we just borrowed (which is now first in current)
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
    /// May cause parent underflow, triggering recursive fixes
    pub fn merge_leaf(&mut self, leaf_rc: Rc<RefCell<BTreeNode<K, V>>>, key: &K) -> bool {

        // MERGE PROCESS

        // When neither sibling can lend a key, we must merge nodes
        //
        // Before:
        //           Parent: [30, 50]
        //           /      |       \
        //       [10,20]  [30]    [50,60]
        //
        // After merging middle with right:
        //           Parent: [30]
        //           /           \
        //       [10,20]      [30,50,60]
        let mut underflowed_parent: Option<_> = None;
        
        let left_sibl = BTreeNode::left_sibling(Rc::clone(&leaf_rc));

        // We are the leftmost node
        if left_sibl.is_none() {
            // No left sibling, so merge with right sibling instead
            // We'll absorb the right sibling's data
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

                    // Merge: move all data from right sibling to current
                    curr_node_data.append(data);
                    

                    // Update linked list pointer
                    if next.is_none() {
                        *curr_next = None;
                    } else {
                        *curr_next = Some(next.as_ref().map(Rc::clone).unwrap());
                    }

                     // Update parent: remove separator and right child pointer
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

        } 
        // We have a left sibling - merge with it
        else if left_sibl.is_some() {
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
    ///
    /// Tries in order:
    /// 1. Borrow from right sibling
    /// 2. Borrow from left sibling
    /// 3. Merge with sibling (may recursively fix parent)
    ///
    /// Special case: if root ends up with only 1 child, make that child the new root
    fn fix_parent_underflow(&mut self, node_rc: Rc<RefCell<BTreeNode<K, V>>>) {
        // SPECIAL CASE: Root with single child
        
        // If the root has no keys and only one child, the tree shrinks in height
        //
        // Before:      Root: [ ]
        //                     |
        //                  Child
        //
        // After:       Child becomes new root
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
                        // This is the root, and it has only one child
                        // Make that child the new root
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

        // Try borrowing from right sibling
        let left = BTreeNode::left_sibling(Rc::clone(&node_rc));

        if left.is_none() {
            // We're the leftmost, try right sibling
            if let Some((right, _)) = BTreeNode::right_sibling(Rc::clone(&node_rc)) {
                // BORROW FROM RIGHT SIBLING (INTERNAL NODE)
                //
                // Before:
                //         Parent: [50, 80]
                //         /       |       \
                //    Current:[20] Right:[60,70]  ...
                //
                // After borrowing:
                //         Parent: [60, 80]  <- updated separator
                //         /       |       \
                //    Current:[20,50] Right:[70]  ...
                //      ^sep pulled down  ^child moved
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
                            // Find separator index in parent
                            sep_idx = p_ch.iter().position(|c| Rc::ptr_eq(c, &node_rc)).unwrap();
                            sep_key = p_keys[sep_idx].clone();
                            // Pull separator down to current node
                            c_keys.push(sep_key);

                            // Move first child from right to current
                            moved_child = r_ch.remove(0);
                            c_ch.push(Rc::clone(&moved_child));

                            // Promote first key of right sibling to parent
                            p_keys[sep_idx] = r_keys.remove(0);
                        } else {
                            return;
                        }
                    }
                    // Update moved child's parent pointer
                    match &mut *moved_child.borrow_mut() {
                        BTreeNode::Leaf { parent, .. } | BTreeNode::Internal { parent, .. } => {
                            *parent = Some(Rc::downgrade(&node_rc));
                        }
                    }

                    return;
                }
            }
        }
        // Try borrowing from left sibling
        if let Some((left_sib, _)) = left {
            if left_sib.borrow().can_borrow(self.order) {
                // BORROW FROM LEFT SIBLING (INTERNAL NODE)
                //
                // Similar to right borrow, but in reverse:
                // - Pull separator down from parent
                // - Take last child from left sibling
                // - Push up left's last key to be new separator
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
        // Case when NEITHER sibling can lend, so we merge
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
            // CASE A: Merge with left sibling (we're not leftmost)
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
            } 
            // CASE B: Merge with right sibling (we're leftmost ( pos == 0 ))
            else {
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


    /// Helper for working with leaf nodes through a closure
    /// Lets you modify leaf data without manual pattern matching everywhere
    ///
    /// CLOSURE-BASED MUTATION PATTERN
    
    /// This is a convenience wrapper that:
    /// 1. Borrows node mutably
    /// 2. Runs your closure with access to leaf fields
    /// 3. Automatically ends borrow when closure completes
    ///
    /// Example usage:
    /// ```ignore
    /// node.as_mut_leaf(|parent, data, next| {
    ///     data.push(entry);  // Modify the data
    ///     *next = Some(...); // Update next pointer
    /// });
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

    /// Read-only version of as_mut_leaf
    /// Provides immutable access to leaf fields through a closure

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

    /// Compares two leaf nodes by their key ranges
    /// Used to verify the linked list ordering in tests
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

    /// Checks if this node has enough keys to lend one to a sibling
    /// Without going below the minimum
    ///
    /// MINIMUM KEY CALCULATION
    
    /// For order N:
    ///   Minimum keys = ceil(N/2) - 1
    ///
    /// Examples:
    ///   order=3: min = ceil(3/2) - 1 = 2 - 1 = 1
    ///   order=4: min = ceil(4/2) - 1 = 2 - 1 = 1
    ///   order=5: min = ceil(5/2) - 1 = 3 - 1 = 2
    ///
    /// A node can borrow if:
    ///   (current_keys - 1) >= minimum_keys
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

    /// Finds the left sibling of this node
    /// Returns the sibling and its position in the parent's children array
    ///
    /// SIBLING FINDING LOGIC
    
    /// To find the left sibling:
    /// 1. Get our parent
    /// 2. Find our position in parent's children array
    /// 3. If position > 0, the node at position-1 is our left sibling
    /// 4. If position == 0, we're the leftmost child (no left sibling)
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

    /// Finds the right sibling of this node
    /// Returns the sibling and its position in the parent's children array
    ///
    /// SIBLING FINDING LOGIC
    
    /// To find the right sibling:
    /// 1. Get our parent
    /// 2. Find our position in parent's children array
    /// 3. If position+1 < children.len(), node at position+1 is our right sibling
    /// 4. Otherwise, we're the rightmost child (no right sibling)
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

/// Iterator implementation to walk through leaf nodes
/// Follows the 'next' pointer from leaf to leaf
///
/// LINKED LIST TRAVERSAL
///
/// B+Trees maintain a linked list of leaf nodes for efficient range scans
///
/// Example:
/// ```ignore
    // let mut current = tree.leftmost_leaf();
    // while let Some(next) = current.next() {
    // Process each leaf in order
    //     current = next;
/// }
/// ```
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
        let mut rng = rand::rng();
        let mut t = BTree::new(5);
        let mut expected = HashSet::new();

        for _ in 0..5000 {
            let op = rng.random_range(0..3);
            let key = rng.random_range(0..500);

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
        let mut rng = rand::rng();
        let mut t = BTree::new(10);
        let mut keys_inserted = HashSet::new();

        // Phase 1: Bulk random insert
        let mut insert_keys: Vec<i32> = (0..10000).collect();
        insert_keys.shuffle(&mut rand::rng());

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
            let key = rng.random_range(0..10000);
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
        keys.shuffle(&mut rand::rng());

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

