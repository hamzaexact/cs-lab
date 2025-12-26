// B+Tree Implementation with In-Depth Documentation
// December 26, 2025
//
// A learning-focused B+Tree implementation in Rust.
// Not fully optimized**
//
// Key characteristics:
// - All data lives in leaf nodes
// - Internal nodes only store keys for navigation
// - Leaf nodes are linked together (like a linked list)
// - Supports insert, search, and delete with automatic rebalancing
//
// Read more about B+Trees: https://en.wikipedia.org/wiki/B%2B_tree

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

/// Main B+Tree structure
///
/// The `order` determines how many keys a node can hold before splitting.
/// For example, order=3 means a node can have at most 3 keys.
pub struct BTree {
    pub root: Option<Rc<RefCell<BTreeNode>>>,
    order: usize,
}

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
/// ```
#[derive(Debug)]
pub enum BTreeNode {
    Internal {
        parent: Option<Weak<RefCell<BTreeNode>>>,
        keys: Vec<i32>,
        children: Vec<Rc<RefCell<BTreeNode>>>,
    },

    Leaf {
        parent: Option<Weak<RefCell<BTreeNode>>>,
        data: Vec<Entry>,
        next: Option<Rc<RefCell<BTreeNode>>>,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LeafNode<'l> {
    // 'l => leaf
    pub parent: &'l mut Option<Weak<RefCell<BTreeNode>>>,
    pub data: &'l mut Vec<Entry>,
    pub next: &'l mut Option<Rc<RefCell<BTreeNode>>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Entry {
    pub key: i32,
    pub data: String,
}

#[allow(dead_code)]
pub enum NodeCmpOrd {
    Less,
}

/// Determines what action to take when deleting from an underflowing node
pub enum DeletePlanner {
    Empty,
    Simple,
    RightBorrow,
    LeftBorrow,
    Merge,
}

impl BTree {
    /// Creates a new empty B+Tree with the specified order
    ///
    /// Example:
    /// ```
    /// let mut tree = BTree::new(5); // order 5 means max 5 keys per node
    /// ```
    pub fn new(order: usize) -> Self {
        Self { root: None, order }
    }

    /// Checks if the tree has no data
    pub fn _is_empty(&self) -> bool {
        // Case 1: No root exists at all
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
    pub fn search(&self, key: i32) -> Option<Entry> {
        // Empty tree check
        if self.root.is_none() {
            return None;
        }

        // Start traversal from root
        let mut current = self.root.as_ref().map(Rc::clone).unwrap();

        // PHASE 1: Navigate down the tree to find the correct leaf
        //
        // This loop continues until we reach a leaf node.
        // At each internal node, we decide which child to follow based on key comparisons.
        let _ = loop {
            let next = {
                let node = current.borrow();
                match &*node {
                    BTreeNode::Internal {
                        parent: _,
                        keys,
                        children,
                    } => {
                        // INTERNAL NODE NAVIGATION
                        //
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

                        let mut chosen = None;

                        // Linear search through keys to find correct child
                        // NOTE: Binary search would be more efficient for large nodes,
                        // but linear search is simpler for educational purposes
                        for (index, num) in keys.iter().enumerate() {
                            if key < *num {
                                // Found first key larger than search key
                                // Take the child to the left of this key
                                chosen = Some(Rc::clone(&children[index]));
                                break;
                            }
                        }

                        // If we didn't find a larger key, the search key is >= all keys
                        // so take the rightmost child
                        chosen.unwrap_or_else(|| Rc::clone(children.last().unwrap()))
                    }
                    BTreeNode::Leaf { .. } => {
                        // Reached a leaf node - exit the loop
                        break;
                    }
                }
            };
            current = next;
        };

        // PHASE 2: Search within the leaf node
        //
        // Now 'current' points to the leaf that should contain our key (if it exists)
        if let BTreeNode::Leaf {
            parent: _,
            data: entries,
            next: _,
        } = &*current.borrow()
        {
            // Linear search through entries in this leaf
            // The entries are sorted by key, so we could use binary search here too
            for entry in entries.iter() {
                if (*entry).key == key {
                    // Found it! Return a clone of the entry
                    return Some(entry.clone());
                }
            }
        }

        // Key not found in the tree
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
    pub fn insert(&mut self, entry: Entry) {
        // STEP 1: Find the appropriate leaf for this key
        //    =
        // This traverses the tree from root to leaf, following the path
        // that the key would take during a search operation
        let leaf = self.find_leaf(entry.key);

        // STEP 2: Insert the entry into the leaf and maintain sorted order
        //    ==
        if let BTreeNode::Leaf {
            parent: _,
            data: keys,
            next: _,
        } = &mut *leaf.borrow_mut()
        {
            // Add the new entry
            keys.push(entry);

            // Sort to maintain the invariant that keys are always in order
            // This makes searches efficient and splits predictable
            keys.sort_by_key(|k| k.key);
        }

        // STEP 3: Check if we need to split due to overflow
        //
        let is_leaf = (*leaf.borrow()).is_leaf();

        if (*leaf.borrow()).is_node_full(self.order) {
            // Node has exceeded capacity (keys.len() > order)
            // Need to split it and re-distribute entries

            if is_leaf {
                // LEAF SPLIT PROCESS:
                //
                // 1. Create a new right sibling leaf
                // 2. Move half the entries to the new leaf
                // 3. Update the linked list pointers (next)
                // 4. Promote the middle key to the parent
                // 5. If parent overflows, split it recursively

                self.split_leaf(Rc::clone(&leaf));

                // Debugging hook (currently commented out)
                match &*leaf.borrow() {
                    BTreeNode::Leaf { .. } => {
                        let current = self.root.as_ref().map(Rc::clone).unwrap();
                        if let BTreeNode::Internal { parent: _, .. } = &*current.borrow() {
                            // Could inspect tree structure here for debugging
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Walks down the tree to find which leaf should contain this key
    /// Creates a new root leaf if tree is empty
    fn find_leaf(&mut self, key: i32) -> Rc<RefCell<BTreeNode>> {
        // CASE 1: Empty tree
        //   =
        // If there's no root, create the first leaf node and make it the root
        if self.root.is_none() {
            let leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                parent: None,
                data: Vec::new(),
                next: None,
            }));
            self.root = Some(Rc::clone(&leaf));
            return leaf;
        }

        // CASE 2: Tree exists - traverse to find the correct leaf
        //      ==
        let mut current = Rc::clone(self.root.as_ref().unwrap());

        // This loop navigates from root to leaf, following the path
        // determined by key comparisons at each internal node
        let leaf = loop {
            let next = {
                let node = current.borrow();

                match &*node {
                    BTreeNode::Leaf { .. } => {
                        // Found our target leaf - this is where the key belongs
                        break Rc::clone(&current); // <-- loop RETURNS this
                    }

                    BTreeNode::Internal { keys, children, .. } => {
                        // NAVIGATION LOGIC
                        //
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

                        let mut chosen = None;

                        for (index, num) in keys.iter().enumerate() {
                            if key < *num {
                                chosen = Some(Rc::clone(&children[index]));
                                break;
                            }
                        }

                        // If key >= all internal keys, go to rightmost child
                        chosen.unwrap_or_else(|| Rc::clone(children.last().unwrap()))
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
        parent: &mut Option<Weak<RefCell<BTreeNode>>>,
    ) -> Weak<RefCell<BTreeNode>> {
        // CASE 1: No parent exists (we're at the root)
        //
        if parent.is_none() {
            // Create a new internal node to become the parent
            let parent = Rc::new(RefCell::new(BTreeNode::new_internal()));

            // Since there's no parent, this node must become the new root
            // This is how the tree grows in height
            self.root = Some(parent);

            return self.root.as_ref().map(Rc::downgrade).unwrap();
        }

        // CASE 2: Parent exists
        //
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
    ///       |      >  |  (linked via 'next')
    /// ```
    ///
    /// The middle key gets promoted to parent
    fn split_leaf(&mut self, leaf: Rc<RefCell<BTreeNode>>) {
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode>>> = None;
        let mut mut_leaf_ptr = leaf.borrow_mut();

        // BORROW SCOPE BEGINS: We have mutable access to the overflowing leaf

        if let BTreeNode::Leaf {
            parent: left_child_parent,
            data: left_keys,
            next,
        } = &mut *mut_leaf_ptr
        {
            // STEP 1: Get or create a parent to hold the split nodes

            // The parent will receive:
            //   - A pointer to the left child (original leaf)
            //   - The separator key
            //   - A pointer to the right child (new leaf we're about to create)

            let ptr_leaf_parent = (self.get_node_parent(left_child_parent).upgrade()).unwrap();
            let mut leaf_parent = ptr_leaf_parent.borrow_mut(); // MUTABLE BORROW

            if let BTreeNode::Internal {
                parent: _,
                keys: parent_keys,
                children: parent_children,
            } = &mut *leaf_parent
            {
                // STEP 2: Calculate split point and the key to promote
                //
                // For a leaf with 4 entries [10, 20, 30, 40]:
                //   - middle = 4 / 2 = 2
                //   - key_to_promote = left_keys[2].key = 30
                //   - Left keeps: [10, 20]
                //   - Right gets: [30, 40]

                let middle = left_keys.len() / 2;
                let key_to_promote = left_keys[middle].key;

                // STEP 3: Create the right sibling leaf
                //
                let right_child = {
                    let tmp = Rc::new(RefCell::new(BTreeNode::Leaf {
                        // Point back to the parent we just obtained
                        parent: Some(Rc::downgrade(&ptr_leaf_parent)),

                        // split_off(middle) removes elements [middle..] from left_keys
                        // and returns them as a new Vec
                        // After this, left_keys contains [0..middle]
                        data: left_keys.split_off(middle),

                        // LINKED LIST MAINTENANCE

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

                // STEP 4: Update pointers to maintain tree structure

                // Left child (original leaf) now points to the parent
                *left_child_parent = Some(Rc::downgrade(&ptr_leaf_parent));

                // Left child's 'next' pointer now points to right sibling
                // This maintains the linked list of leaves for range scans
                *next = Some(Rc::clone(&right_child));

                // STEP 5: Insert children and key into parent

                if parent_children.is_empty() {
                    // SPECIAL CASE: Root split
                    //
                    // This is the first split ever. The parent is empty and just became root.
                    // Tree structure before:  [10, 20, 30, 40] (single leaf, is root)
                    // Tree structure after:        [30]
                    //                             /    \
                    //                        [10,20]  [30,40]

                    parent_children.push(Rc::clone(&leaf)); // Left child
                    parent_keys.push(key_to_promote); // Separator
                    parent_children.push(Rc::clone(&right_child)); // Right child
                } else {
                    // NORMAL CASE: Parent already has children
                    //       -
                    // Find where the left (original) leaf is in parent's children array
                    let left_index = parent_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &leaf))
                        .unwrap();

                    // Insert the separator key at the appropriate position
                    // Example: if left_index=1, parent keys [20, 40, 60]
                    //          inserting 30 at position 1 gives [20, 30, 40, 60]
                    parent_keys.insert(left_index, key_to_promote);

                    // Insert right child immediately after left child
                    parent_children.insert(left_index + 1, Rc::clone(&right_child));
                }

                // STEP 6: Check if parent overflowed from this insertion
                //      =
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

        // STEP 7: Handle cascading splits
        //
        // If the parent overflowed, we need to split it too
        // This can cascade all the way up to the root
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
    fn split_internal(&mut self, node: Rc<RefCell<BTreeNode>>) {
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode>>> = None;

        // POINTER STRUCTURE DURING SPLIT
        //    ==
        //
        // We're splitting an internal node that looks like this:
        //
        //              PARENT
        //                 |
        //                 v
        //         LEFT NODE (to split)
        //        /  |  |  \   (has too many children)
        //       /   |  |   \
        //      C0  C1 C2   C3
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

        let mut mut_left_ptr = node.borrow_mut();
        if let BTreeNode::Internal {
            parent: left_node_parent,
            keys: left_node_keys,
            children: left_node_children,
        } = &mut *mut_left_ptr
        {
            // STEP 1: Get or create parent
            //
            let parent = self.get_node_parent(left_node_parent).upgrade().unwrap();
            let mut mut_parent_ptr = parent.borrow_mut();

            // STEP 2: Determine split point
            //    =
            // For keys [3, 5, 7, 9] with order=4:
            //   - middle = 4 / 2 = 2
            //   - key_to_promote = left_node_keys[2] = 7
            //   - After split:
            //       Left gets:  [3, 5]
            //       Right gets: [9]
            //       Parent gets: 7 (the separator)

            let middle = left_node_keys.len() / 2;
            let key_to_promote = left_node_keys[middle];

            // STEP 3: Create right sibling internal node
            //   =
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

            // STEP 4: Update left node's parent pointer
            //
            *left_node_parent = Some(Rc::downgrade(&parent));

            // STEP 5: Fix parent pointers for left node's children
            //
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

            // STEP 6: Fix parent pointers for right node's children
            //
            let mut mut_right_ptr = right_node.borrow_mut();
            if let BTreeNode::Internal {
                parent: _,
                keys: _,
                children: right_node_children,
            } = &mut *mut_right_ptr
            {
                for child in right_node_children.iter_mut() {
                    let mut child_node = child.borrow_mut();

                    // Update parent for internal children
                    if let BTreeNode::Internal {
                        parent: child_parent,
                        ..
                    } = &mut *child_node
                    {
                        *child_parent = Some(Rc::downgrade(&right_node));
                    }

                    // Update parent for leaf children using helper function
                    if let Some(_) = (*child_node).as_mut_leaf(|parent, _, _| {
                        *parent = Some(Rc::downgrade(&right_node));
                    }) {}
                }
            }

            // Release borrow early to prevent conflicts
            drop(mut_right_ptr);

            // STEP 7: Insert split results into parent
            //
            if let BTreeNode::Internal {
                parent: _,
                keys: parent_keys,
                children: parent_children,
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
                    //
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

                // Check if parent overflowed
                if parent_keys.len() >= self.order {
                    overflowed_parent = Some(Rc::clone(&parent));
                }
            }

            drop(mut_parent_ptr);
            drop(mut_left_ptr);
        }

        // STEP 8: Handle cascading splits
        //
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
    pub fn delete(&mut self, key: i32) -> bool {
        // OVERVIEW OF DELETION
        //
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

        // STEP 1: Find the leaf containing the key
        //
        let leaf_rc = self.find_leaf(key);

        // STEP 2: Special case - deleting from root
        //
        if Rc::ptr_eq(&leaf_rc, self.root.as_ref().unwrap()) {
            // The leaf IS the root (tree has only one node)
            let state = leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                let index = data.iter().position(|entry| entry.key == key);
                if index.is_none() {
                    return false;
                }
                // Remove the key directly - no underflow concerns for root
                data.remove(index.unwrap());
                return true;
            });

            if state.unwrap() {
                return true;
            } else {
                return false;
            }
        }

        // STEP 3: Plan the deletion strategy
        //
        // This analyzes the node and its siblings to determine:
        //   - Can we delete without underflow? (Simple)
        //   - Can we borrow from right sibling? (RightBorrow)
        //   - Can we borrow from left sibling? (LeftBorrow)
        //   - Must we merge? (Merge)
        let plan = self.delete_planner(key, Rc::clone(&leaf_rc));

        // STEP 4: Execute the deletion plan
        //
        match plan {
            (DeletePlanner::Empty, ..) => {
                // Tree is empty or key doesn't exist
                return false;
            }

            (DeletePlanner::Simple, ..) => {
                // SIMPLE DELETION
                //
                // Node will still have enough keys after deletion
                // Just remove the key directly
                leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                    let index = data.iter().position(|entry| entry.key == key);
                    if index.is_none() {
                        return false;
                    }
                    data.remove(index.unwrap());
                    return true;
                });
            }

            (DeletePlanner::RightBorrow, r_leaf, pos) => {
                // BORROW FROM RIGHT
                //
                // Right sibling has extra keys
                // Move one key from right sibling to current node
                return self.right_borrow(Rc::clone(&leaf_rc), key, r_leaf, pos);
            }

            (DeletePlanner::LeftBorrow, left_sibl, left_sibl_pos) => {
                // BORROW FROM LEFT
                //
                // Left sibling has extra keys
                // Move one key from left sibling to current node
                return self.left_borrow(key, leaf_rc, left_sibl, left_sibl_pos);
            }

            _ => {
                // MERGE CASE
                //
                // Neither sibling can lend a key
                // Must merge this node with a sibling
                return self.merge_leaf(leaf_rc, key);
            }
        }

        false
    }

    /// Decides which deletion strategy to use based on the tree state
    /// Returns the plan along with sibling info if needed
    fn delete_planner(
        &mut self,
        _: i32,
        leaf_rc: Rc<RefCell<BTreeNode>>,
    ) -> (DeletePlanner, Rc<RefCell<BTreeNode>>, usize) {
        // DELETION PLANNING ALGORITHM
        //   ==
        // This function determines the safest way to delete a key
        // without violating B+Tree properties
        //
        // Priority order:
        //   1. Check if tree is empty
        //   2. Check if simple deletion works (node stays valid)
        //   3. Try borrowing from right sibling
        //   4. Try borrowing from left sibling
        //   5. Fall back to merge

        // CHECK 1: Is this an empty root?
        //
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

        // CHECK 2: Can we do a simple delete?
        //
        // Simple delete works if: (keys_after_delete) >= ceil(order/2) - 1
        //
        // Example with order=5:
        //   Minimum keys = ceil(5/2) - 1 = 2
        //   If node has 3 keys, deleting 1 leaves 2 (OK!)
        //   If node has 2 keys, deleting 1 leaves 1 (UNDERFLOW!)

        let state = leaf_rc.borrow().can_borrow(self.order);
        if state {
            return (DeletePlanner::Simple, Rc::clone(&leaf_rc), 0x00);
        }

        // CHECK 3: Can we borrow from right sibling?
        //   =
        // Borrowing works if the sibling has more than the minimum keys
        //
        // Example with order=5 (min=2 keys):
        //   Current:  [10]      (will underflow after delete)
        //   Right:    [30, 40, 50]  (has 3 > 2, can lend one!)
        //
        // After borrowing:
        //   Current:  [30]
        //   Right:    [40, 50]

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

        // CHECK 4: Can we borrow from left sibling?
        //
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
        //
        // If we can't borrow from either sibling, we must merge
        (DeletePlanner::Merge, Rc::clone(&leaf_rc), 0x03)
    }

    /// Handles delete by borrowing a key from the right sibling
    /// Also updates the parent separator key
    pub fn right_borrow(
        &mut self,
        leaf: Rc<RefCell<BTreeNode>>,
        deleted_key: i32,
        right_sibl: Rc<RefCell<BTreeNode>>,
        right_sibling_pos: usize,
    ) -> bool {
        // RIGHT BORROW PROCESS
        //
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
            // STEP 1: Delete the target key
            //    =
            let pos = data.iter().position(|e| e.key == deleted_key);
            if pos.is_none() {
                return false;
            }
            data.remove(pos.unwrap());

            // STEP 2: Borrow first entry from right sibling
            //
            if let BTreeNode::Leaf { data: sib_dat, .. } = &mut *right_sibl.borrow_mut() {
                // Remove the first entry from right sibling
                let entry = sib_dat.remove(0);

                // Add it to current node
                data.push(entry);

                // STEP 3: Update parent separator key
                //
                // The separator key between current and right sibling
                // must now be the first key of the right sibling
                let new_sep = sib_dat[0].key;

                let prnt = parent.as_ref().unwrap().upgrade().unwrap();
                if let BTreeNode::Internal { keys, .. } = &mut *prnt.borrow_mut() {
                    // Update the separator at position right_sibling_pos - 1
                    keys[right_sibling_pos - 1] = new_sep;
                }
            }
        }
        true
    }

    /// Handles delete by borrowing a key from the left sibling
    /// Also updates the parent separator key
    pub fn left_borrow(
        &mut self,
        key: i32,
        leaf_rc: Rc<RefCell<BTreeNode>>,
        left_sibl: Rc<RefCell<BTreeNode>>,
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
            // STEP 1: Delete the target key
            //    =
            let key_to_delete = data.iter().position(|e| e.key == key);
            if key_to_delete.is_none() {
                return false;
            }
            data.remove(key_to_delete.unwrap());

            // STEP 2: Borrow last entry from left sibling

            if let BTreeNode::Leaf { data: left_dat, .. } = &mut *left_sibl.borrow_mut() {
                let last_idx = left_dat.len() - 1;
                let t_key = left_dat[last_idx].key;

                // Remove last from left, insert at beginning of current
                data.insert(0, left_dat.remove(last_idx));

                // STEP 3: Update parent separator
                //
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
    pub fn merge_leaf(&mut self, leaf_rc: Rc<RefCell<BTreeNode>>, key: i32) -> bool {
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

        // CASE 1: We are the leftmost node
        //
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
                    // Delete the target key first
                    let k_to_rm = curr_node_data.iter().position(|e| e.key == key);
                    if k_to_rm.is_none() {
                        return false;
                    }
                    curr_node_data.remove(k_to_rm.unwrap());

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
                        keys.remove(0); // Remove first separator
                        children.remove(1); // Remove right sibling pointer

                        // Check if parent underflowed
                        if (self.order as f32 / 2_f32).ceil() - 1_f32 > keys.len() as f32 {
                            underflowed_parent = Some(Rc::clone(&t_prnt));
                        }
                    }
                    drop(t_prnt);
                }
                drop(leaf_tmp_brr);
            }
            drop(tmp_r_borrow);

            // Handle parent underflow recursively
            if underflowed_parent.is_some() {
                self.fix_parent_underflow(underflowed_parent.unwrap());
            }
        }
        // CASE 2: We have a left sibling - merge with it
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
                    // Delete target key
                    let k_to_rm = curr_node_data.iter().position(|e| e.key == key);
                    if k_to_rm.is_none() {
                        return false;
                    }
                    curr_node_data.remove(k_to_rm.unwrap());

                    // Merge: move current's data to left sibling
                    data.append(curr_node_data);

                    // Update linked list
                    if curr_next.is_none() {
                        *next = None
                    } else {
                        let new_next = Rc::clone(&curr_next.as_ref().unwrap());
                        *next = Some(new_next);
                    }

                    // Update parent
                    let t_prnt = l_prnt.as_ref().unwrap().upgrade().unwrap();
                    if let BTreeNode::Internal { keys, children, .. } = &mut *t_prnt.borrow_mut() {
                        keys.remove(curr_left_sib_pos);
                        children.remove(curr_left_sib_pos + 1);

                        // Check for underflow
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
    fn fix_parent_underflow(&mut self, node_rc: Rc<RefCell<BTreeNode>>) {
        // SPECIAL CASE: Root with single child
        //
        // If the root has no keys and only one child, the tree shrinks in height
        //
        // Before:      Root: [ ]
        //                     |
        //                   Child
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
                                *parent = None; // Child is now root, has no parent
                            }
                        }
                        self.root = Some(child);
                    }
                    return;
                }
            }
        }

        // STRATEGY 1: Try borrowing from right sibling
        //
        let left = BTreeNode::left_sibling(Rc::clone(&node_rc));

        if left.is_none() {
            // We're the leftmost, try right sibling
            if let Some((right, _)) = BTreeNode::right_sibling(Rc::clone(&node_rc)) {
                if right.borrow().can_borrow(self.order) {
                    // BORROW FROM RIGHT SIBLING (INTERNAL NODE)
                    //
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

                    let sep_idx;
                    let parent_rc;

                    // Get parent reference
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
                            sep_key = p_keys[sep_idx];

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

        // STRATEGY 2: Try borrowing from left sibling

        if let Some((left_sib, _)) = left {
            if left_sib.borrow().can_borrow(self.order) {
                // BORROW FROM LEFT SIBLING (INTERNAL NODE)
                //
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

                        // Pull separator down to front of current
                        c_keys.insert(0, p_keys[sep_idx]);

                        // Move last child from left to front of current
                        moved_child = l_ch.pop().unwrap();
                        c_ch.insert(0, Rc::clone(&moved_child));

                        // Push up left's last key
                        p_keys[sep_idx] = l_keys.pop().unwrap();
                    } else {
                        return;
                    }
                }

                // Update moved child's parent
                match &mut *moved_child.borrow_mut() {
                    BTreeNode::Leaf { parent, .. } | BTreeNode::Internal { parent, .. } => {
                        *parent = Some(Rc::downgrade(&node_rc));
                    }
                }

                return;
            }
        }

        // STRATEGY 3: Must merge with sibling
        //
        // Neither sibling can lend, so we merge

        let parent_rc = {
            let node = node_rc.borrow();
            if let BTreeNode::Internal { parent, .. } = &*node {
                parent.as_ref().unwrap().upgrade().unwrap()
            } else {
                return;
            }
        };

        let mut parent = parent_rc.borrow_mut();

        // Find our position in parent
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
            //
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
                        // Pull separator key down into left node
                        left_keys.push(parent_keys.remove(pos - 1));

                        // Merge all keys and children from current to left
                        left_keys.append(curr_keys);
                        left_children.append(curr_children);

                        // Remove current node from parent
                        parent_children.remove(pos);

                        // Update all children to point to left_sibling
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
            // CASE B: Merge with right sibling (we're leftmost)
            //
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
                        // Pull separator down into current
                        curr_keys.push(parent_keys.remove(0));

                        // Merge right into current
                        curr_keys.append(right_keys);
                        curr_children.append(right_children);

                        // Remove right sibling
                        parent_children.remove(1);

                        // Update children
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

        // Check if parent underflowed and recurse if needed
        if let BTreeNode::Internal { keys, .. } = &*parent {
            if keys.len() < ((self.order + 1) / 2) - 1 {
                drop(parent);
                self.fix_parent_underflow(parent_rc);
            }
        }
    }

    /// Prints the tree level-by-level for debugging
    /// (I) means Internal node, (L) means Leaf node
    pub fn _print_tree(&self) {
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
                        let keys: Vec<_> = data.iter().map(|e| e.key).collect();
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
    pub fn _leftmost_leaf(&mut self) -> Option<Rc<RefCell<BTreeNode>>> {
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
            BTreeNode::Internal { .. } => {
                return Some(self.find_leaf(i32::MIN));
            }
        }
    }
}

impl BTreeNode {
    fn is_leaf(&self) -> bool {
        if let BTreeNode::Leaf {
            parent: _,
            data: _,
            next: _,
        } = self
        {
            return true;
        }
        false
    }

    /// Checks if this node has exceeded its capacity
    fn is_node_full(&self, order: usize) -> bool {
        match self {
            Self::Leaf {
                parent: _,
                data: keys,
                next: _,
            } => {
                if keys.len() > order {
                    return true;
                }
            }
            Self::Internal {
                parent: _,
                keys,
                children: _,
            } => {
                if keys.len() > order {
                    return true;
                }
            }
        }

        false
    }

    fn new_internal() -> BTreeNode {
        BTreeNode::Internal {
            parent: None,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    fn _new_leaf() -> BTreeNode {
        BTreeNode::Leaf {
            parent: None,
            data: Vec::new(),
            next: None,
        }
    }

    pub fn _as_raw_leaf(&mut self) -> Option<LeafNode<'_>> {
        match self {
            BTreeNode::Leaf { parent, data, next } => return Some(LeafNode { parent, data, next }),

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
    /// ```
    /// node.as_mut_leaf(|parent, data, next| {
    ///     data.push(entry);  // Modify the data
    ///     *next = Some(...); // Update next pointer
    /// });
    /// ```
    fn as_mut_leaf<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(
            &mut Option<Weak<RefCell<BTreeNode>>>,
            &mut Vec<Entry>,
            &mut Option<Rc<RefCell<BTreeNode>>>,
        ) -> R,
    {
        match self {
            Self::Leaf { parent, data, next } => {
                // Call the closure with mutable references to all leaf fields
                let result = f(parent, data, next);
                return Some(result);
            }
            _ => return None,
        }
    }

    /// Read-only version of as_mut_leaf
    /// Provides immutable access to leaf fields through a closure
    pub fn _as_ref_leaf<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(
            &Option<Weak<RefCell<BTreeNode>>>,
            &Vec<Entry>,
            &Option<Rc<RefCell<BTreeNode>>>,
        ) -> R,
    {
        match self {
            Self::Leaf { parent, data, next } => {
                let res = f(parent, data, next);
                return Some(res);
            }
            _ => None,
        }
    }

    /// Compares two leaf nodes by their key ranges
    /// Used to verify the linked list ordering in tests
    pub fn _cmp(&mut self, other: &mut BTreeNode, ord: NodeCmpOrd) -> bool {
        // Extract leaf data from both nodes
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

        // Compare based on ordering type
        match ord {
            NodeCmpOrd::Less => {
                // Check if ALL keys in 'a' are less than ALL keys in 'b'
                // This verifies proper ordering in the linked list
                return a.data.last().as_ref().unwrap().key < b.data.first().as_ref().unwrap().key;
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
            BTreeNode::Leaf {
                parent: _,
                data,
                next: _,
            } => {
                // Check if removing one entry still keeps us above minimum
                (data.len() - 1) as f32 >= (order as f32 / 2.0).ceil() - 1.0
            }
            BTreeNode::Internal { keys, .. } => {
                // Same logic for internal nodes
                (keys.len() - 1) as f32 >= (order as f32 / 2.0).ceil() - 1.0
            }
        }
    }

    /// Finds the left sibling of this node
    /// Returns the sibling and its position in the parent's children array
    ///
    /// SIBLING FINDING LOGIC
    ///  
    /// To find the left sibling:
    /// 1. Get our parent
    /// 2. Find our position in parent's children array
    /// 3. If position > 0, the node at position-1 is our left sibling
    /// 4. If position == 0, we're the leftmost child (no left sibling)
    pub fn left_sibling(
        node_rc: Rc<RefCell<BTreeNode>>,
    ) -> Option<(Rc<RefCell<BTreeNode>>, usize)> {
        // CASE 1: Node is a leaf
        //  =
        if node_rc.borrow().is_leaf() {
            if let BTreeNode::Leaf { parent, .. } = &*node_rc.borrow() {
                // No parent means this is the root - no siblings
                if parent.is_none() {
                    return None;
                }

                let curr_nd_prnt = parent.as_ref().unwrap().upgrade().unwrap();

                if let BTreeNode::Internal {
                    parent: _,
                    children: prnt_children,
                    ..
                } = &*curr_nd_prnt.borrow()
                {
                    // Find our position in parent's children
                    let pos = prnt_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &node_rc));

                    // If we're at position 0, we're leftmost
                    if let Some(0) = pos {
                        return None;
                    }

                    // Position not found means error
                    if pos.is_none() {
                        return None;
                    }

                    // Return left sibling and its position
                    return Some((
                        Rc::clone(&prnt_children[pos.unwrap() - 1]),
                        pos.unwrap() - 1,
                    ));
                }
            }
        };

        // CASE 2: Node is internal
        //
        if let BTreeNode::Internal { parent, .. } = &*node_rc.borrow() {
            // Check if parent exists and is valid
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

                // Leftmost child has no left sibling
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
    ///  
    /// To find the right sibling:
    /// 1. Get our parent
    /// 2. Find our position in parent's children array
    /// 3. If position+1 < children.len(), node at position+1 is our right sibling
    /// 4. Otherwise, we're the rightmost child (no right sibling)
    pub fn right_sibling(
        node_rc: Rc<RefCell<BTreeNode>>,
    ) -> Option<(Rc<RefCell<BTreeNode>>, usize)> {
        // CASE 1: LEAF NODE
        //
        if let BTreeNode::Leaf { parent, .. } = &*node_rc.borrow() {
            // Use ? operator for clean error handling
            // Returns None if parent doesn't exist or upgrade fails
            let parent = parent.as_ref()?.upgrade()?;

            if let BTreeNode::Internal { children, .. } = &*parent.borrow() {
                // Find our position
                let pos = children
                    .iter()
                    .position(|child| Rc::ptr_eq(child, &node_rc))?;

                // Right sibling exists only if index + 1 < children.len()
                if pos + 1 < children.len() {
                    return Some((Rc::clone(&children[pos + 1]), pos + 1));
                } else {
                    return None; // We're rightmost
                }
            }
        }

        // CASE 2: INTERNAL NODE
        //
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

/// B+Trees maintain a linked list of leaf nodes for efficient range scans
///
/// Example:
/// ```
/// let mut current = tree.leftmost_leaf();
/// while let Some(next) = current.next() {
///     // Process each leaf in order
///     current = next;
/// }
/// ```
impl Iterator for BTreeNode {
    type Item = Rc<RefCell<BTreeNode>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Leaf {
                parent: _, next, ..
            } => match next {
                Some(c_next) => {
                    // Clone the Rc to return ownership of the next node
                    let next = Rc::clone(c_next);
                    return Some(next);
                }
                None => return None, // End of linked list
            },
            _ => None, // Internal nodes don't participate in iteration
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, panic};

    use rand::seq::SliceRandom;

    use super::*;

    fn e(k: i32) -> Entry {
        Entry {
            key: k,
            data: k.to_string(),
        }
    }

    #[test]
    fn test_sorted_insert() {
        let mut t = BTree::new(128);

        for i in 0..1_000_000 {
            t.insert(Entry {
                key: i,
                data: format!("{}", i),
            });
        }

        for i in 0..1_000_000 {
            let res = t.search(i);
            assert!(res.is_some(), "Missing key {}", i);
        }
    }

    #[test]
    fn test_random_insert() {
        let mut t = BTree::new(4);

        let mut v: Vec<_> = (0..1_000_000).collect();
        v.shuffle(&mut rand::rng());

        for i in &v {
            t.insert(Entry {
                key: *i,
                data: format!("{}", i),
            });
        }

        for i in 0..100_000 {
            assert!(t.search(i).is_some(), "Missing key {}", i);
        }
    }

    #[test]
    fn test_linked_leaves() {
        let mut t = BTree::new(64);
        let mut v: Vec<_> = (0..10000).collect();
        v.shuffle(&mut rand::rng());
        for i in 0..10000 {
            t.insert(Entry {
                key: i,
                data: format!("{}", i),
            });
        }

        let leftmost_leaf = t._leftmost_leaf();
        match leftmost_leaf {
            Some(mut current_node) => loop {
                let next = {
                    let curr = &mut *current_node.borrow_mut();
                    if let Some(node) = curr.next() {
                        if curr._cmp(&mut *node.borrow_mut(), NodeCmpOrd::Less) == true {
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

        // Build tree
        for k in [10, 20, 30, 40, 50] {
            t.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        // Delete without underflow
        t.delete(10);

        // Remaining keys must exist
        for k in [20, 30, 40, 50] {
            assert!(t.search(k).is_some(), "Key {} missing", k);
        }

        // Deleted key must be gone
        assert!(t.search(10).is_none());
    }

    #[test]
    fn delete_triggers_right_borrow() {
        let mut t = BTree::new(5);

        // Force structure that will underflow on delete
        for k in [10, 20, 30, 40, 50, 60, 70] {
            t.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        // Delete to cause underflow in left leaf
        t.delete(10);
        t.delete(20);

        // Validate all remaining keys exist
        for k in [30, 40, 50, 60, 70] {
            assert!(t.search(k).is_some(), "Key {} missing", k);
        }

        // Deleted keys must be gone
        assert!(t.search(10).is_none());
        assert!(t.search(20).is_none());
    }
    #[test]
    fn delete_simple_no_underflow_2() {
        let mut t = BTree::new(3);

        for k in [10, 20, 30, 40] {
            t.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        // Delete does NOT cause underflow
        t.delete(20);

        assert!(t.search(20).is_none());
        assert!(t.search(10).is_some());
        assert!(t.search(30).is_some());
        assert!(t.search(40).is_some());
    }
    #[test]
    fn delete_triggers_right_borrow_only() {
        let mut t = BTree::new(3);

        // This layout is intentional
        for k in [10, 20, 30, 40, 50, 60] {
            t.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        // Causes underflow in left leaf, right sibling has extra
        t.delete(10);
        t.delete(20);

        // Tree must still contain remaining keys
        for k in [30, 40, 50, 60] {
            assert!(t.search(k).is_some(), "Missing key {}", k);
        }

        assert!(t.search(10).is_none());
        assert!(t.search(20).is_none());
    }
    #[test]
    fn delete_triggers_left_borrow_only() {
        let mut t = BTree::new(3);

        for k in [10, 20, 30, 40, 50, 60, 70, 80] {
            t.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        // Shape is now stable
        t.delete(30); // simple
        t.delete(50); // simple
        t.delete(40); // left borrow

        for k in [10, 20, 60, 70, 80] {
            assert!(t.search(k).is_some(), "Missing key {}", k);
        }

        for k in [30, 40, 50] {
            assert!(t.search(k).is_none());
        }
    }
    #[test]
    fn mixed_simple_and_borrow_deletes_no_merge() {
        let mut t = BTree::new(3);

        for k in [10, 20, 30, 40, 50, 60, 70] {
            t.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        t.delete(10); // right borrow
        t.delete(40); // simple
        t.delete(60); // simple

        for k in [20, 30, 50, 70] {
            assert!(t.search(k).is_some());
        }

        for k in [10, 40, 60] {
            assert!(t.search(k).is_none());
        }
    }
    fn build_tree_order3() -> BTree {
        let mut tree = BTree::new(3);
        for k in [10, 20, 30, 40, 50, 60] {
            tree.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }
        tree
    }
    fn _build_tree_order5() -> BTree {
        let mut tree = BTree::new(5);
        for k in (10..=80).step_by(5) {
            tree.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }
        tree
    }
    #[test]
    fn delete_simple_no_underflow_3() {
        let mut tree = build_tree_order3();

        tree.delete(60);
        assert!(tree.search(60).is_none());

        for k in [10, 20, 30, 40, 50] {
            assert!(tree.search(k).is_some());
        }
    }
    #[test]
    fn delete_simple_multiple() {
        let mut tree = build_tree_order3();

        tree.delete(20);
        tree.delete(30);

        assert!(tree.search(20).is_none());
        assert!(tree.search(30).is_none());

        for k in [10, 40, 50, 60] {
            assert!(tree.search(k).is_some());
        }
    }
    #[test]
    fn delete_triggers_right_borrow_2() {
        let mut tree = BTree::new(3);

        for k in [10, 20, 30, 40, 50, 60] {
            tree.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        tree.delete(10);
        tree.delete(20);

        assert!(tree.search(10).is_none());
        assert!(tree.search(20).is_none());

        for k in [30, 40, 50, 60] {
            assert!(tree.search(k).is_some());
        }
    }
    #[test]
    fn right_borrow_updates_parent_separator() {
        let mut tree = BTree::new(5);

        for k in [10, 20, 25, 30, 40, 45, 50] {
            tree.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        tree.delete(10);

        for k in [20, 25, 30, 40, 45, 50] {
            assert!(tree.search(k).is_some());
        }
    }
    #[test]
    fn left_borrow_preserves_search() {
        let mut tree = BTree::new(5);

        for k in [10, 15, 20, 25, 30, 35, 40] {
            tree.insert(Entry {
                key: k,
                data: k.to_string(),
            });
        }

        tree.delete(40);
        tree.delete(35);

        for k in [10, 15, 20, 25, 30] {
            assert!(tree.search(k).is_some());
        }
    }
    #[test]
    fn delete_nonexistent_key() {
        let mut tree = build_tree_order3();

        tree.delete(999);

        for k in [10, 20, 30, 40, 50, 60] {
            assert!(tree.search(k).is_some());
        }
    }
    #[test]
    fn delete_and_reinsert() {
        let mut tree = build_tree_order3();

        tree.delete(30);
        assert!(tree.search(30).is_none());

        tree.insert(Entry {
            key: 30,
            data: "30".to_string(),
        });

        assert!(tree.search(30).is_some());
    }
    #[test]
    fn many_simple_deletes() {
        let mut tree = BTree::new(5);
        let n = 100;
        for i in 0..n {
            tree.insert(Entry {
                key: i,
                data: i.to_string(),
            });
        }

        for i in (0..n).step_by(3) {
            tree.delete(i);
        }

        for i in (0..n).step_by(3) {
            assert!(tree.search(i).is_none());
        }

        for i in (1..n).step_by(3) {
            assert!(tree.search(i).is_some());
        }
    }

    #[test]
    fn test_insert_delete_insert_same_key() {
        let mut tree = BTree::new(5);

        tree.insert(e(50));
        assert!(tree.search(50).is_some());

        tree.delete(50);
        assert!(tree.search(50).is_none());

        tree.insert(e(50));
        assert!(tree.search(50).is_some());

        tree.delete(50);
        tree.insert(e(50));
        assert!(tree.search(50).is_some());
    }

    #[test]
    fn test_alternating_insert_delete_corrected() {
        let mut tree = BTree::new(3);

        let mut expected = std::collections::HashSet::new();

        for i in 0..20 {
            tree.insert(Entry {
                key: i,
                data: i.to_string(),
            });
            expected.insert(i);

            if i > 0 && i % 2 == 0 {
                tree.delete(i - 1);
                expected.remove(&(i - 1));
            }
        }

        // Verify final state exactly
        for i in 0..20 {
            assert_eq!(
                tree.search(i).is_some(),
                expected.contains(&i),
                "Mismatch for key {}",
                i
            );
        }
    }

    #[test]
    fn test_delete_causes_multiple_merges() {
        let mut tree = BTree::new(3);

        for i in 0..20 {
            tree.insert(e(i));
        }

        for i in (0..10).rev() {
            tree.delete(i);
        }

        for i in 10..20 {
            assert!(tree.search(i).is_some());
        }

        for i in 0..10 {
            assert!(tree.search(i).is_none());
        }
    }
    #[test]
    fn test_delete_causes_root_shrink() {
        let mut tree = BTree::new(3);

        for i in 0..15 {
            tree.insert(e(i));
        }

        for i in 0..12 {
            tree.delete(i);
        }

        assert!(tree.search(12).is_some());
        assert!(tree.search(13).is_some());
        assert!(tree.search(14).is_some());
    }
    #[test]
    fn test_minimum_order_3() {
        let mut tree = BTree::new(3);

        for i in 0..100 {
            tree.insert(e(i));
        }

        for i in 0..100 {
            assert!(tree.search(i).is_some());
        }

        for i in (0..50).rev() {
            tree.delete(i);
        }

        for i in 50..100 {
            assert!(tree.search(i).is_some());
        }
    }

    #[test]
    fn test_large_order_100() {
        let mut tree = BTree::new(100);

        for i in 0..1000 {
            tree.insert(e(i));
        }

        for i in 0..1000 {
            assert!(tree.search(i).is_some());
        }
    }

    #[test]
    fn test_single_key_operations() {
        let mut tree = BTree::new(5);

        tree.insert(e(42));
        assert!(tree.search(42).is_some());

        tree.delete(42);
        assert!(tree.search(42).is_none());

        tree.insert(e(100));
        assert!(tree.search(100).is_some());
    }

    #[test]
    fn test_sequential_insert_sequential_delete() {
        let mut tree = BTree::new(5);
        let n = 1000;

        for i in 0..n {
            tree.insert(e(i));
        }

        for i in 0..n {
            assert!(tree.search(i).is_some());
        }

        for i in 0..n {
            tree.delete(i);
            assert!(tree.search(i).is_none());
        }
    }

    #[test]
    fn test_random_insert_random_delete() {
        let mut tree = BTree::new(7);
        let mut keys = HashSet::new();

        for i in 0..500 {
            let key = (i * 97 + 31) % 1000;
            tree.insert(e(key));
            keys.insert(key);
        }

        for &key in &keys {
            assert!(tree.search(key).is_some());
        }

        let to_delete: Vec<_> = keys.iter().step_by(2).copied().collect();
        for &key in &to_delete {
            tree.delete(key);
            keys.remove(&key);
        }

        for &key in &to_delete {
            assert!(tree.search(key).is_none());
        }

        for &key in &keys {
            assert!(tree.search(key).is_some());
        }
    }

    #[test]
    fn test_insert_reverse_order() {
        let mut tree = BTree::new(5);

        for i in (0..100).rev() {
            tree.insert(e(i));
        }

        for i in 0..100 {
            assert!(tree.search(i).is_some());
        }
    }

    #[test]
    fn test_delete_reverse_order() {
        let mut tree = BTree::new(5);

        for i in 0..100 {
            tree.insert(e(i));
        }

        for i in (0..100).rev() {
            tree.delete(i);
            assert!(tree.search(i).is_none());
        }
    }

    #[test]
    fn test_delete_from_empty_tree() {
        let mut tree = BTree::new(5);

        tree.delete(42);

        tree.insert(e(42));
        assert!(tree.search(42).is_some());
    }

    #[test]
    fn test_delete_nonexistent_keys() {
        let mut tree = BTree::new(5);

        for i in (0..100).step_by(2) {
            tree.insert(e(i));
        }

        for i in (1..100).step_by(2) {
            tree.delete(i);
        }

        for i in (0..100).step_by(2) {
            assert!(tree.search(i).is_some());
        }
    }

    #[test]
    fn beast_delete_only_search_based() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        use std::collections::HashSet;

        const N: usize = 50_000;

        let mut rng = StdRng::seed_from_u64(0xCAFEBABE);
        let mut tree = BTree::new(64);
        let mut alive = HashSet::<i32>::new();

        // ---- INSERT PHASE ----
        for i in 0..N as i32 {
            tree.insert(Entry {
                key: i,
                data: i.to_string(),
            });
            alive.insert(i);
        }
        let mut keys: Vec<i32> = alive.iter().copied().collect();
        keys.shuffle(&mut rng);

        for (step, k) in keys.iter().enumerate() {
            tree.delete(*k);
            alive.remove(k);

            // Check deleted key is gone
            assert!(
                tree.search(*k).is_none(),
                "Deleted key {} still exists at step {}",
                k,
                step
            );

            // Check some random surviving keys
            for _ in 0..5 {
                if alive.is_empty() {
                    break;
                }
                let idx = rng.random_range(0..alive.len());
                let test_key = *alive.iter().nth(idx).unwrap();

                assert!(
                    tree.search(test_key).is_some(),
                    "Existing key {} missing at step {}",
                    test_key,
                    step
                );
            }
        }

        for i in 0..N as i32 {
            assert!(
                tree.search(i).is_none(),
                "Tree not empty at end, key {} still exists",
                i
            );
        }
    }
}
