use std::{
    cell::RefCell,
    ptr,
    rc::{Rc, Weak},
};

pub struct BTree {
    root: Option<Rc<RefCell<BTreeNode>>>,
    order: usize,
}

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
pub struct LeafNode<'l> {
    // 'l => leaf
    pub parent: &'l mut Option<Weak<RefCell<BTreeNode>>>,
    pub data: &'l mut Vec<Entry>,
    pub next: &'l mut Option<Rc<RefCell<BTreeNode>>>,
}

#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub key: i32,
    pub data: String,
}

pub enum NodeCmpOrd {
    Less,
    Greater,
}

impl BTree {
    pub fn new(order: usize) -> Self {
        Self { root: None, order }
    }
    //
    //
    //
    // TODO: SEARCH IMPLEMENTATION
    //
    //
    //
    pub fn search(&self, key: i32) -> Option<Entry> {
        if self.root.is_none() {
            return None;
        }

        let mut current = self.root.as_ref().map(Rc::clone).unwrap();
        let entry = loop {
            let next = {
                let node = current.borrow();
                match &*node {
                    BTreeNode::Internal {
                        parent: _,
                        keys: keys,
                        children: children,
                    } => {
                        // ELIMINATE CANDIDATES BASED ON MODULES OPERATOR AND THEIR LENGTH
                        // OR USE BINARY search
                        //
                        let mut chosen = None;

                        for (index, num) in keys.iter().enumerate() {
                            if key < *num {
                                chosen = Some(Rc::clone(&children[index]));
                                break;
                            }
                        }

                        chosen.unwrap_or_else(|| Rc::clone(children.last().unwrap()))
                    }

                    BTreeNode::Leaf {
                        parent: _,
                        data: entires,
                        next,
                    } => {
                        break;
                    }
                }
            };
            current = next;
        };

        if let BTreeNode::Leaf {
            parent: _,
            data: entries,
            next: _,
        } = &*current.borrow()
        {
            for entry in entries.iter() {
                if (*entry).key == key {
                    return Some(entry.clone());
                }
            }
        }

        None
    }

    ///
    ///
    ///
    ///
    /// TODO: INSERT IMPLEMENTATION
    ///
    ///
    ///

    pub fn insert(&mut self, entry: Entry) {
        let mut leaf = self.find_leaf(entry.key);
        let tmp = entry.clone();
        if let BTreeNode::Leaf {
            parent: _,
            data: keys,
            next: _,
        } = &mut *leaf.borrow_mut()
        {
            keys.push(entry);
            // DEBUG:
            // println!("key with value {} INSERTED into {:?}", tmp.key, keys);
            keys.sort_by_key(|k| k.key);
        }
        let is_leaf = (*leaf.borrow()).is_leaf();
        if (*leaf.borrow()).is_node_full(self.order) {
            // Need to split it and re-distribute.
            if is_leaf {
                self.split_leaf(Rc::clone(&leaf));
                match &*leaf.borrow() {
                    BTreeNode::Leaf {
                        parent: pare,
                        data: d,
                        next: _,
                    } => {
                        let current = self.root.as_ref().map(Rc::clone).unwrap();
                        if let BTreeNode::Internal {
                            parent: _,
                            keys: keys,
                            children: children,
                        } = &*current.borrow()
                        {
                            // if keys.len() >= 3 {
                            //     println!("{:#?}", children);
                            //     println!("{:#?}", keys);
                            // }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn find_leaf(&mut self, key: i32) -> Rc<RefCell<BTreeNode>> {
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

        // Loop returns the leaf node
        let leaf = loop {
            let next = {
                let node = current.borrow();

                match &*node {
                    BTreeNode::Leaf { .. } => {
                        break Rc::clone(&current); // <-- loop RETURNS this
                    }

                    BTreeNode::Internal { keys, children, .. } => {
                        let mut chosen = None;

                        for (index, num) in keys.iter().enumerate() {
                            if key < *num {
                                chosen = Some(Rc::clone(&children[index]));
                                break;
                            }
                        }

                        chosen.unwrap_or_else(|| Rc::clone(children.last().unwrap()))
                    }
                }
            };

            current = next;
        };

        leaf
    }

    fn get_node_parent(
        &mut self,
        parent: &mut Option<Weak<RefCell<BTreeNode>>>,
    ) -> Weak<RefCell<BTreeNode>> {
        if parent.is_none() {
            let parent = Rc::new(RefCell::new(BTreeNode::new_internal()));
            // since there is no parent == no root;
            self.root = Some(parent);
            return self.root.as_ref().map(Rc::downgrade).unwrap();
        }

        // println!("RETURNING FROM HERE");
        ((parent.as_ref().unwrap()).upgrade())
            .as_ref()
            .map(Rc::downgrade)
            .unwrap()
    }

    fn split_leaf(&mut self, leaf: Rc<RefCell<BTreeNode>>) {
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode>>> = None;
        let mut mut_leaf_ptr = leaf.borrow_mut();

        // BORROW START HERE IN ACTION
        if let BTreeNode::Leaf {
            parent: left_child_parent,
            data: left_keys,
            next: next,
        } = &mut *mut_leaf_ptr
        // INSIDE THE GIVEN LEAF NODE THAT HAS THE OVER FLOW
        {
            // WE MAKE A PARENT TO HOLD THEM
            let ptr_leaf_parent = (self.get_node_parent(left_child_parent).upgrade()).unwrap();
            let mut leaf_parent = ptr_leaf_parent.borrow_mut(); // MUTED
            // BORROW HERE
            if let BTreeNode::Internal {
                parent: _,
                keys: parent_keys,
                children: parent_children,
            } = &mut *leaf_parent
            {
                let middle = left_keys.len() / 2;
                let key_to_promote = left_keys[middle].key;

                // AFTER GETTING THE PARENT I CREATE A RIGHT CHILD TO HOLD THE REMINING NODES
                let right_child = {
                    let tmp = Rc::new(RefCell::new(BTreeNode::Leaf {
                        parent: Some(Rc::downgrade(&ptr_leaf_parent)), // let it point to the
                        // parent
                        data: left_keys.split_off(middle), // REMINING NODES
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

                let mut_right_ptr = right_child.borrow_mut();
                *left_child_parent = Some(Rc::downgrade(&ptr_leaf_parent));
                *next = Some(Rc::clone(&right_child));
                if parent_children.is_empty() {
                    // Root case: first split ever
                    parent_children.push(Rc::clone(&leaf));
                    parent_keys.push(key_to_promote);
                    parent_children.push(Rc::clone(&right_child));
                } else {
                    // Find where the left (original) leaf is in parent's children
                    let left_index = parent_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &leaf))
                        .unwrap();
                    // Insert the promoted key at the position
                    parent_keys.insert(left_index, key_to_promote);

                    // Insert right child immediately after left child
                    parent_children.insert(left_index + 1, Rc::clone(&right_child));

                    // update left and right pointers
                    //
                    //
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

    fn split_internal(&mut self, node: Rc<RefCell<BTreeNode>>) {
        let mut overflowed_parent: Option<Rc<RefCell<BTreeNode>>> = None;

        // NODE: the Node that we want to split
        //
        //
        //
        // INITIALIZATION :
        //
        // LEFT NODE<---+
        //     |        |
        //     |        |
        //     |        |
        //     +        |
        // PARENT NODE<-+ <-- LIFETIME
        //     |        |
        //     |        |
        //     |        |
        //     +        |
        // RIGHT NODE<--+
        //
        //
        //
        //

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
            let key_to_promote = left_node_keys[middle];
            let right_node = Rc::new(RefCell::new(BTreeNode::Internal {
                parent: Some(Rc::downgrade(&parent)),
                keys: {
                    let keys = left_node_keys.split_off(middle + 1);
                    // If keys are [3, 5, 7, 9]:
                    // Right keys: [9]
                    // Left keys: [3, 5, 7]
                    // Since this is an internal split, the middle key (7) should not be included. We will remove it here.
                    left_node_keys.pop();
                    keys
                },
                children: left_node_children.split_off(middle + 1),
            }));

            // UPDATE THE LEFT NODE PARENT POINTER

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

            // FIX THE RIGHT NODE POINTER
            let mut mut_right_ptr = right_node.borrow_mut();
            if let BTreeNode::Internal {
                parent: _,
                keys: _,
                children: right_node_children,
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
                    // if let Some(leaf) = (*child_node).as_leaf() {
                    //     *leaf.parent = Some(Rc::downgrade(&right_node));
                    // }

                    if let Some(res) = (*child_node).as_mut_leaf(|parent, _, _| {
                        *parent = Some(Rc::downgrade(&right_node));
                    }) {}
                    // *k.parent = Some(Rc::downgrade(&right_node));
                    // println!("KPARENT BEFORE: {:?}", *k.parent);
                    // if let BTreeNode::Leaf {
                    //     parent: child_parent,
                    //     ..
                    // } = &mut *child_node
                    // {
                    //     *child_parent = Some(Rc::downgrade(&right_node));
                    //                     println!("KPARENT after: {:?}", *child_parent);
                    //
                    // }
                }
            }
            // DROP IT HERE MANUALLY TO PREVENT ANY FUTURE BORROWING ISSUES
            drop(mut_right_ptr);

            if let BTreeNode::Internal {
                parent: _,
                keys: parent_keys,
                children: parent_children,
            } = &mut *mut_parent_ptr
            {
                if parent_children.is_empty() {
                    // Root case
                    parent_children.push(Rc::clone(&node));
                    parent_keys.push(key_to_promote);
                    parent_children.push(Rc::clone(&right_node));
                } else {
                    // Find where the left node is in parent's children
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
        } // END OF THE LEFT NODE SCOOP

        if overflowed_parent.is_some() {
            self.split_internal(overflowed_parent.unwrap());
        }
    }

    ///
    ///
    ///
    ///
    /// TODO: DELETE IMPLEMENTATION
    ///
    ///
    ///

    pub fn delete(&mut self, key: i32) -> bool {
        // Return the leaf that contains the key we're trying to delete.
        let leaf_rc = self.find_leaf(key);
        // Simple check for the basic case, where a leaf node has sufficient parent_children
        let state = leaf_rc
            .borrow()
            .as_ref_leaf(|_, keys, _| {
                ((keys.len() - 1) as f32 >= ((self.order as f32 / 2_f32).round() - 1.0))
            })
            .unwrap(); // Unwrap because find_leaf  function MUST RETURN A LEAF

        // Overflow if we executed this block
        if !state {}


        // TODO: Simple case
        // I switched this to true to test the behaviour underflow!
        if true {
            leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                ({
                    let index = data.iter().position(|entry| entry.key == key);
                    if index.is_none() {
                        return false;
                    }
                    data.remove(index.unwrap());
                    return true;
                })
            });
        }

        false
    }

    /// TODO: DEBUG IMPLEMENTATION
    pub fn print_tree(&self) {
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

                    BTreeNode::Leaf { data, next, .. } => {
                        // Print leaf node keys
                        let keys: Vec<_> = data.iter().map(|e| e.key).collect();
                        print!("(L [{:?}]) ", keys);
                    }
                }
            }

            println!();
            level += 1;
        }
        println!("---------------");
    }

    pub fn leftmost_leaf(&mut self) -> Option<Rc<RefCell<BTreeNode>>> {
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
            BTreeNode::Internal { keys: keys, .. } => {
                let min_key = keys.iter().min().unwrap();
                let k = (*self.find_leaf(i32::MIN).borrow_mut()).as_raw_leaf();
                return Some((self.find_leaf(i32::MIN)));
            }
        }
        None
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

    fn is_node_full(&self, order: usize) -> bool {
        match self {
            Self::Leaf {
                parent: _,
                data: keys,
                next: _,
            } => {
                if keys.len() > order {
                    // FOR DEBUG
                    // println!("OVERFLOW AT LEAF -> {:?}", keys);
                    return true;
                }
            }
            Self::Internal {
                parent: _,
                keys: keys,
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

    fn new_leaf() -> BTreeNode {
        BTreeNode::Leaf {
            parent: None,
            data: Vec::new(),
            next: None,
        }
    }

    pub fn as_raw_leaf(&mut self) -> Option<LeafNode<'_>> {
        match self {
            BTreeNode::Leaf {
                parent: parent,
                data: data,
                next: next,
            } => return Some(LeafNode { parent, data, next }),

            _ => None,
        }
    }

    // This is a simple approach to matching every time.
    // Borrow node mutably  ──────────────┐
    //                                    │
    //      +-----------------------+     │
    //      | run your closure here | <───┘  (mutation allowed)
    //      +-----------------------+
    //                                    │
    // Borrow ends automatically here ────┘
    //
    //
    //
    fn as_mut_leaf<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(
            &mut Option<Weak<RefCell<BTreeNode>>>,
            &mut Vec<Entry>,
            &mut Option<Rc<RefCell<BTreeNode>>>,
        ) -> R,
    {
        match self {
            Self::Leaf {
                parent: parent,
                data: data,
                next: next,
            } => {
                let result = f(parent, data, next);
                return Some(result);
            }
            _ => return None,
        }
    }

    pub fn as_ref_leaf<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(
            &Option<Weak<RefCell<BTreeNode>>>,
            &Vec<Entry>,
            &Option<Rc<RefCell<BTreeNode>>>,
        ) -> R,
    {
        match self {
            Self::Leaf {
                parent: parent,
                data: data,
                next: next,
            } => {
                let res = f(parent, data, next);
                return Some(res);
            }
            _ => None,
        }
    }

    pub fn cmp(&mut self, other: &mut BTreeNode, ord: NodeCmpOrd) -> bool {
        let mut a;
        let mut b;
        match self.as_raw_leaf() {
            Some(leaf) => {
                a = leaf;
            }
            _ => return false,
        }

        match other.as_raw_leaf() {
            Some(leaf) => {
                b = leaf;
            }
            _ => return false,
        }

        match ord {
            NodeCmpOrd::Greater => {
                return a.data.last().as_ref().unwrap().key > b.data.first().as_ref().unwrap().key;
            }

            NodeCmpOrd::Less => {
                return a.data.last().as_ref().unwrap().key < b.data.first().as_ref().unwrap().key;
            }

            _ => unreachable!(),
        }

        false
    }
}

impl Iterator for BTreeNode {
    type Item = Rc<RefCell<BTreeNode>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Leaf {
                parent: _,
                data,
                next,
            } => match next {
                Some(c_next) => {
                    let next = Rc::clone(c_next);
                    return Some(next);
                }
                None => return None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use rand::seq::SliceRandom;

    use super::*;

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
        v.shuffle(&mut rand::thread_rng());

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
        v.shuffle(&mut rand::thread_rng());
        for i in 0..10000 {
            t.insert(Entry {
                key: i,
                data: format!("{}", i),
            });
        }

        let mut leftmost_leaf = t.leftmost_leaf();
        match leftmost_leaf {
            Some(mut current_node) => loop {
                let next = {
                    let curr = &mut *current_node.borrow_mut();
                    if let Some(node) = curr.next() {
                        if curr.cmp(&mut *node.borrow_mut(), NodeCmpOrd::Less) == true {
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
}
