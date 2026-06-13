// B+Tree Implementation
// December 26, 2025
//
// *Not fully optimized
//
// All data lives in leaf nodes
// * Internal nodes only store keys for navigation
// * Leaf nodes are linked together (like a linked list)
// * Supports insert, search, and delete with automatic rebalancing
//
// Read more about B+Trees: https://en.wikipedia.org/wiki/B%2B_tree
#![allow(clippy::empty_line_after_doc_comments)]
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
///
/// A node in the B+Tree - either Internal (for navigation) or Leaf (stores data)
#[derive(Debug)]
pub enum BTreeNode<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    /// Internal nodes guide searches but don't store data
    /// They contain keys for routing and pointers to child nodes
    Internal {
        /// Weak pointer to parent node (prevents reference cycles)
        ///  
        /// None if this is the root
        parent: Option<Weak<RefCell<BTreeNode<K, V>>>>,

        /// Separator keys used for navigation
        /// If keys = [20, 40], then: child[0] has keys <20, child[1] has 20-40, child[2] has ≥40
        /// Must have: keys.len() + 1 == children.len()
        keys: Vec<K>,

        /// Pointers to child nodes (can be Internal or Leaf)
        /// Always has one more child than keys: N keys → N+1 children
        children: Vec<Rc<RefCell<BTreeNode<K, V>>>>,
    },

    /// Leaf nodes store the actual data entries
    /// All leaves form a sorted linked list via 'next' pointers
    Leaf {
        /// Weak pointer to parent Internal node
        /// None if this leaf is the root (single-node tree)
        parent: Option<Weak<RefCell<BTreeNode<K, V>>>>,

        /// The actual data entries, sorted by key
        /// This is where all tree data lives - internal nodes never store data
        data: Vec<Entry<K, V>>,

        /// Pointer to next leaf in the linked list (for range scans)
        /// None if this is the rightmost leaf
        next: Option<Rc<RefCell<BTreeNode<K, V>>>>,
    },
}
// K?
/// Used to avoid repetitive pattern matching when working with leaves
#[derive(Debug)]
#[allow(dead_code)]
pub struct LeafNode<'l, K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    /// Mutable reference to the parent pointer
    pub parent: &'l mut Option<Weak<RefCell<BTreeNode<K, V>>>>,

    /// Mutable reference to the data entries
    pub data: &'l mut Vec<Entry<K, V>>,

    /// Mutable reference to the next-leaf pointer
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
    /// The search key used for ordering and lookups
    pub key: K,

    /// The data (V) payload associated with this key
    /// In a real database, this might be a row ID or serialized record
    pub val: V,
}

/// Comparison ordering for comparing two nodes
/// Currently only supports checking if one node's keys are all less than another's
#[allow(dead_code)]
pub enum NodeCmpOrd {
    /// Check if all keys in first node < all keys in second node
    /// Used to verify linked list ordering in tests
    Less,
}

/// Strategy enum returned by delete_planner() to handle different deletion scenarios
/// Determines what action to take when deleting from a node that might underflow
pub enum DeletePlanner {
    /// Tree is empty or key doesn't exist - do nothing
    Empty,

    /// Node has enough keys after deletion - just remove the key directly
    /// Happens when: (keys_after_delete) ≥ ⌈order/2⌉ - 1
    Simple,

    /// Node will underflow, but right sibling can lend a key
    /// Move one key from right sibling and update parent separator
    RightBorrow,

    /// Node will underflow, but left sibling can lend a key
    /// Move one key from left sibling and update parent separator
    LeftBorrow,

    /// Node will underflow and siblings can't help - must merge with a sibling
    /// May cause parent to underflow, triggering recursive fixes
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
    /// ``` ignore
    // let mut tree = BTree::new(5); // order 5 means max 5 keys per node
    //
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
        loop {
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
        }

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
        if let Some(ref overflowed_parent) = overflowed_parent {
            self.split_internal(Rc::clone(overflowed_parent));
        }
        // *died-code
        // if overflowed_parent.is_some() {
        //     // If the parent overflowed, we need to split it too
        //     // This can cascade all the way up to the root
        //     self.split_internal(overflowed_parent.unwrap());
        // }
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
        /*
                           +-----------+
                       +---+  Parent   |
                       |   +-----------+
                       |
                       |
                       |
                       |
                   +---------+
                   | LEFT    | (to Split; has too many children)
              +----+   NODE  +--+
              |    -+----+---+  |
              |     |    |      |
              |     |    |      |
              |     |    |      |
              |     |    |      |
            +---+ +---+ +---+  +---+
            |C1 | |C2 | |C3 |  |C4 |
            +---+ +---+ +---+  +---+
        */
        //
        //
        //
        //
        // After split:
        /*
                            +-----------+
                     +------+  PARENT   +------+
                     |      +-----------+      |
                     |                         |
                     |                         |
                     |                         |
                     v                         v
                 +--------+                +-------+
              +--+ LEFT   +-+           +--+ RIGHT +--+
              |  +--------+ |           |  +-------+  |
              |             |           |             |
              |             |           |             |
            +---+         +---+       +---+         +---+
            |C1 |         |C2 |       |C3 |         |C4 |
            +---+         +---+       +---+         +---+

        */

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
                };
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
                };
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
                };
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
                };
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
            };
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
            };
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
            };
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
                Some(c_next) => Some(Rc::clone(c_next)),
                None => None,
            },
            _ => None,
        }
    }
}
