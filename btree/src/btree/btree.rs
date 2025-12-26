use std::{
    cell::{RefCell},
    rc::{Rc, Weak},
};

pub struct BTree {
    pub root: Option<Rc<RefCell<BTreeNode>>>,
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
#[allow(dead_code)]
pub struct LeafNode<'l> {
    // 'l => leaf
    pub parent: &'l mut Option<Weak<RefCell<BTreeNode>>>,
    pub data: &'l mut Vec<Entry>,
    pub next: &'l mut Option<Rc<RefCell<BTreeNode>>>,
}


#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct Entry {
    pub key: i32,
    pub data: String,
}

#[allow(dead_code)]
pub enum NodeCmpOrd {
    Less,
}

pub enum DeletePlanner {
    Empty,
    Simple,
    RightBorrow,
    LeftBorrow,
    Merge,
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
    pub fn _is_empty(&self) -> bool {
        if self.root.is_none() {
            return true;
        }
        if let BTreeNode::Leaf { data, .. } = &*self.root.as_ref().unwrap().borrow() {
            return data.is_empty();
        }
        false
    }
    pub fn search(&self, key: i32) -> Option<Entry> {
        if self.root.is_none() {
            return None;
        }

        let mut current = self.root.as_ref().map(Rc::clone).unwrap();
        let _ = loop {
            let next = {
                let node = current.borrow();
                match &*node {
                    BTreeNode::Internal {
                        parent: _,
                        keys,
                        children,
                    } => {
                        let mut chosen = None;
                        //
                        // NOTE: I should have used binary search here,
                        // but for simplicity's sake, I used a linear search instead.
                        //
                        for (index, num) in keys.iter().enumerate() {
                            if key < *num {
                                chosen = Some(Rc::clone(&children[index]));
                                break;
                            }
                        }
                        chosen.unwrap_or_else(|| Rc::clone(children.last().unwrap()))
                    }
                    BTreeNode::Leaf {
                        ..
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
        let  leaf = self.find_leaf(entry.key);
        if let BTreeNode::Leaf {
            parent: _,
            data: keys,
            next: _,
        } = &mut *leaf.borrow_mut()
        {
            keys.push(entry);
            keys.sort_by_key(|k| k.key);
        }
        let is_leaf = (*leaf.borrow()).is_leaf();
        if (*leaf.borrow()).is_node_full(self.order) {
            // Need to split it and re-distribute.
            if is_leaf {
                self.split_leaf(Rc::clone(&leaf));
                match &*leaf.borrow() {
                    BTreeNode::Leaf {
                        ..
                    } => {
                        let current = self.root.as_ref().map(Rc::clone).unwrap();
                        if let BTreeNode::Internal {
                            parent: _,
                            ..
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
            next,
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

                    if let Some(_) = (*child_node).as_mut_leaf(|parent, _, _| {
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

    /// DONE
    pub fn delete(&mut self, key: i32) -> bool {
        // Get Plan from DeletePlanner
        //
        //
        // Return the leaf that contains the key we're trying to delete.
        let leaf_rc = self.find_leaf(key);
        // Check if its the root
        if Rc::ptr_eq(&leaf_rc, self.root.as_ref().unwrap()) {
            let state = leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                {
                    let index = data.iter().position(|entry| entry.key == key);
                    if index.is_none() {
                        return false;
                    }
                    data.remove(index.unwrap());
                    return true;
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
                //  println!("EMPTY TREE");
                return false;
            }
            (DeletePlanner::Simple, ..) => {
                // println!("SIMPLE PLAN");
                leaf_rc.borrow_mut().as_mut_leaf(|_, data, _| {
                    {
                        let index = data.iter().position(|entry| entry.key == key);
                        if index.is_none() {
                            return false;
                        }
                        data.remove(index.unwrap());
                        return true;
                    }
                });
            }

            (DeletePlanner::RightBorrow, r_leaf, pos) => {
                //  println!("RIGHT PLAN");
                return self.right_borrow(Rc::clone(&leaf_rc), key, r_leaf, pos);
            }

            (DeletePlanner::LeftBorrow, left_sibl, left_sibl_pos) => {
                // println!("LEFT PLAN");
                return self.left_borrow(key, leaf_rc, left_sibl, left_sibl_pos);
            }

            _ => {
                // println!("MERGE PLAN");
                return self.merge_leaf(leaf_rc, key);
            }
        }

        false
    }

    fn delete_planner(
        &mut self,
        _: i32,
        leaf_rc: Rc<RefCell<BTreeNode>>,
    ) -> (DeletePlanner, Rc<RefCell<BTreeNode>>, usize) {
        // TODO: Check If its Empty
        //

        if let BTreeNode::Internal {
            parent,
            keys,
            ..
        } = &*leaf_rc.borrow()
        {
            // ITS A ROOT
            if parent.is_none() {
                if keys.is_empty() {
                    return (DeletePlanner::Empty, Rc::clone(&leaf_rc), 0x00);
                }
            }
        }

        let tmp_borrow = leaf_rc.borrow();

        if let BTreeNode::Leaf { parent, data, .. } = &*tmp_borrow {
            // ITS A ROOT
            if parent.is_none() {
                if data.is_empty() {
                    return (DeletePlanner::Empty, Rc::clone(&leaf_rc), 0x00);
                }
            }
        }
        drop(tmp_borrow);

        //
        // TODO: Check the case for a direct removal scenario.
        //
        //
        // Return the leaf that contains the key we're trying to delete.
        // Simple check for the basic case, where a leaf node has sufficient parent_children
        //
        let state = leaf_rc.borrow().can_borrow(self.order);

        if state {
            return (DeletePlanner::Simple, Rc::clone(&leaf_rc), 0x00);
        }

        // TODO: Check if we can do right borrow
        // To make this work, we must first check
        // if the last key of the current node is
        // smaller than the most right key of its
        // parent node. If so, we can then check
        // if the right sibling has enough keys to borrow.
        // if not we check for the left borrow.

        //
        // __*If there's no next node, we're the rightmost node.

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

        // Since we couldn't borrow from the right node,
        // let's check the left one (if it exists).
        //
        // We can check if we're not the leftmost node by comparing
        // the first key of the current node with the first key of the parent,
        // if the parent key is greater than the current's first key.
        // If so, we're the leftmost.
        // Example:
        //
        //        [20 | 40]
        //         /      \
        //        /        \
        //   [10, 15]   [20, 30, 40]
        //
        //
        // The first key to compare would be 20, since it's the first key at
        // the parent node. We see they're equal, which is correct. However,
        // if the key was 10, then 10 < 20, indicating we are the leftmost child node.
        //
        // NOTE: Since I'm not using this for any project,
        // I'd like to use a quick trick to check if we are the leftmost node. 
        // NOTE:
        // (THAT TRICK WERE REMOVED WHILE DEBUGGING)
        //
        // We match the current leaf to the leftmost leaf using a helper function.
        //
        //
        //

        let left_sibl = BTreeNode::left_sibling(Rc::clone(&leaf_rc));
        if left_sibl.is_some() {
            let (left, pos) = left_sibl.unwrap();
            if left.borrow().can_borrow(self.order) {
                return (DeletePlanner::LeftBorrow, left, pos);
            }
        }

        (DeletePlanner::Merge, Rc::clone(&leaf_rc), 0x03)
    }

    pub fn right_borrow(
        &mut self,
        leaf: Rc<RefCell<BTreeNode>>,
        deleted_key: i32,
        right_sibl: Rc<RefCell<BTreeNode>>,
        right_sibling_pos: usize,
    ) -> bool {
        if let BTreeNode::Leaf { parent, data, .. } = &mut *leaf.borrow_mut() {
            let pos = data.iter().position(|e| e.key == deleted_key);
            if pos.is_none() {
                return false;
            }
            data.remove(pos.unwrap());

            if let BTreeNode::Leaf { data: sib_dat, .. } = &mut *right_sibl.borrow_mut() {
                let entry = sib_dat.remove(0);
                data.push(entry);

                let new_sep = sib_dat[0].key;

                let prnt = parent.as_ref().unwrap().upgrade().unwrap();
                if let BTreeNode::Internal { keys, .. } = &mut *prnt.borrow_mut() {
                    keys[right_sibling_pos - 1] = new_sep;
                }
            }
        }
        true
    }

    pub fn left_borrow(
        &mut self,
        key: i32,
        leaf_rc: Rc<RefCell<BTreeNode>>,
        left_sibl: Rc<RefCell<BTreeNode>>,
        left_sibl_pos: usize,
    ) -> bool {
        if let BTreeNode::Leaf { parent, data, .. } = &mut *leaf_rc.borrow_mut() {
            let key_to_delete = data.iter().position(|e| e.key == key);
            if key_to_delete.is_none() {
                return false;
            }
            data.remove(key_to_delete.unwrap());

            if let BTreeNode::Leaf { data: left_dat, .. } = &mut *left_sibl.borrow_mut() {
                let last_idx = left_dat.len() - 1;
                let t_key = left_dat[last_idx].key;
                data.insert(0, left_dat.remove(last_idx));

                // update parent
                let prnt_tmp = parent.as_ref().unwrap().upgrade().unwrap();
                let mut prnt = prnt_tmp.borrow_mut();
                if let BTreeNode::Internal { keys: pr_k, .. } = &mut *prnt {
                    pr_k[left_sibl_pos] = t_key;
                }
            }
        }

        true
    }

    pub fn merge_leaf(&mut self, leaf_rc: Rc<RefCell<BTreeNode>>, key: i32) -> bool {
        // println!("\n*INSIDE MERGE LEAF FUNCTION*\n");
        let mut underflowed_parent: Option<_> = None;
        let left_sibl = BTreeNode::left_sibling(Rc::clone(&leaf_rc));
        // We are the left most
        if left_sibl.is_none() {
            // Delete the key
            // Merge with right
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
                    let k_to_rm = curr_node_data.iter().position(|e| e.key == key);
                    if k_to_rm.is_none() {
                        return false;
                    }
                    curr_node_data.remove(k_to_rm.unwrap());
                    curr_node_data.append(data);

                    if next.is_none() {
                        *curr_next = None;
                    } else {
                        *curr_next = Some(next.as_ref().map(Rc::clone).unwrap());
                    }

                    let t_prnt = l_prnt.as_ref().unwrap().upgrade().unwrap();
                    if let BTreeNode::Internal { keys, children, .. } = &mut *t_prnt.borrow_mut() {
                        keys.remove(0);
                        children.remove(1); // since we are the left most, index 1 represent
                        // the sibling
                        if (self.order as f32 / 2_f32).ceil() - 1_f32 > keys.len() as f32 {
                            // TODO: PARENT OVERFLOW
                            underflowed_parent = Some(Rc::clone(&t_prnt));
                        }
                    }
                    drop(t_prnt);
                }
                drop(leaf_tmp_brr);
            }
            drop(tmp_r_borrow);
            if underflowed_parent.is_some() {
                //      println!("UNDERFLOW");
                self.fix_parent_underflow(underflowed_parent.unwrap());
            }
        }
        // __LOC
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
                    let k_to_rm = curr_node_data.iter().position(|e| e.key == key);
                    if k_to_rm.is_none() {
                        return false;
                    }
                    curr_node_data.remove(k_to_rm.unwrap());
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
                //   println!("UNDERFLOW WHILE MERGING WITH LEFT / RI");
                self.fix_parent_underflow(underflowed_parent.unwrap());
            }
        }
        true
    }


    fn fix_parent_underflow(&mut self, node_rc: Rc<RefCell<BTreeNode>>) {
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
                            sep_key = p_keys[sep_idx];

                            c_keys.push(sep_key);
                            moved_child = r_ch.remove(0);
                            c_ch.push(Rc::clone(&moved_child));

                            p_keys[sep_idx] = r_keys.remove(0);
                        } else {
                            return;
                        }
                    }

                    // fix moved child parent
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

                        c_keys.insert(0, p_keys[sep_idx]);
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
            } else {
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
                        // move separator key down
                        left_keys.push(parent_keys.remove(pos - 1));

                        // merge keys & children
                        left_keys.append(curr_keys);
                        left_children.append(curr_children);

                        // remove current node pointer
                        parent_children.remove(pos);

                        // fix child parents
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
                        // move separator key down
                        curr_keys.push(parent_keys.remove(0));

                        // merge right into current
                        curr_keys.append(right_keys);
                        curr_children.append(right_children);

                        // remove right sibling
                        parent_children.remove(1);

                        // fix child parents
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

        // recurse if parent underflows
        if let BTreeNode::Internal { keys, .. } = &*parent {
            if keys.len() < ((self.order + 1) / 2) - 1 {
                drop(parent);
                self.fix_parent_underflow(parent_rc);
            }
        }
    }
    /// TODO: DEBUG IMPLEMENTATION
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

                    BTreeNode::Leaf { data,  .. } => {
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
            BTreeNode::Leaf {
                parent,
                data,
                next,
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
                parent,
                data,
                next,
            } => {
                let result = f(parent, data, next);
                return Some(result);
            }
            _ => return None,
        }
    }

    pub fn _as_ref_leaf<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(
            &Option<Weak<RefCell<BTreeNode>>>,
            &Vec<Entry>,
            &Option<Rc<RefCell<BTreeNode>>>,
        ) -> R,
    {
        match self {
            Self::Leaf {
                parent,
                data,
                next,
            } => {
                let res = f(parent, data, next);
                return Some(res);
            }
            _ => None,
        }
    }

    pub fn _cmp(&mut self, other: &mut BTreeNode, ord: NodeCmpOrd) -> bool {
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
                return a.data.last().as_ref().unwrap().key < b.data.first().as_ref().unwrap().key;
            }

        }

    }

    pub fn can_borrow(&self, order: usize) -> bool {
        match self {
            BTreeNode::Leaf {
                parent: _,
                data,
                next: _,
            } => (data.len() - 1) as f32 >= (order as f32 / 2.0).ceil() - 1.0,
            BTreeNode::Internal {
                keys,
                ..
            } => (keys.len() - 1) as f32 >= (order as f32 / 2.0).ceil() - 1.0,
        }
    }

    pub fn left_sibling(
        node_rc: Rc<RefCell<BTreeNode>>,
    ) -> Option<(Rc<RefCell<BTreeNode>>, usize)> {
        if node_rc.borrow().is_leaf() {
            if let BTreeNode::Leaf { parent, .. } = &*node_rc.borrow() {
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
                    // println!("{:#?}", prnt_children);
                    let pos = prnt_children
                        .iter()
                        .position(|child| Rc::ptr_eq(child, &node_rc));
                    if let Some(0) = pos {
                        return None;
                    }
                    // means we are the left_most
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

        if let BTreeNode::Internal {
            parent,
            ..
        } = &*node_rc.borrow()
        {
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
        node_rc: Rc<RefCell<BTreeNode>>,
    ) -> Option<(Rc<RefCell<BTreeNode>>, usize)> {
        // Case 1: LEAF NODE
        if let BTreeNode::Leaf { parent, .. } = &*node_rc.borrow() {
            let parent = parent.as_ref()?.upgrade()?;

            if let BTreeNode::Internal { children, .. } = &*parent.borrow() {
                let pos = children
                    .iter()
                    .position(|child| Rc::ptr_eq(child, &node_rc))?;

                // right sibling exists only if index + 1 < children.len()
                if pos + 1 < children.len() {
                    return Some((Rc::clone(&children[pos + 1]), pos + 1));
                } else {
                    return None;
                }
            }
        }

        // Case 2: INTERNAL NODE
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

impl Iterator for BTreeNode {
    type Item = Rc<RefCell<BTreeNode>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Leaf {
                parent: _,
                next,
                ..
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

        let  leftmost_leaf = t._leftmost_leaf();
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

    // ============================================
    // BOUNDARY TESTS
    // ============================================

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

    // ============================================
    // STRESS TESTS
    // ============================================

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

    // ============================================
    // EDGE CASES
    // ============================================

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

    // ============================================
    // FINAL INTEGRATION TEST
    // ============================================

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

        // ---- RANDOM DELETE PHASE ----
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

        // ---- FINAL TREE MUST BE EMPTY ----
        for i in 0..N as i32 {
            assert!(
                tree.search(i).is_none(),
                "Tree not empty at end, key {} still exists",
                i
            );
        }
    }
}
