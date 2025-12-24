#![allow(warnings)]
mod btree;
use btree::btree::{BTree, BTreeNode, Entry, LeafNode, NodeCmpOrd};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::{random, *};
use std::cell::RefCell;
use std::clone;
use std::rc::Rc;
use std::time::Instant;

fn main() {
    let mut t = BTree::new(4); // realistic B+Tree order for performance
    let n = 10;
    let mut rng = rand::thread_rng();
    let mut keys: Vec<i32> = (0..n as i32).collect();

    // Shuffle for worst-case insert pattern
    keys.shuffle(&mut rng);
    // println!("INSERTING {} keys...", n);
    let start_insert = Instant::now();

    for &k in &keys {
        t.insert(Entry {
            key: k,
            data: k.to_string(),
        });
    }
    //
    // // ------------------------------
    // // SEARCH BENCHMARK
    // // ------------------------------
    // println!("SEARCHING {} random keys...", n);
    // let start_search = Instant::now();
    //
    // for _ in 0..n {
    //     let k = rng.gen_range(0..n as i32);
    //     let res = t.search(k);
    //     assert!(res.is_some());
    // }
    // let search_time = start_search.elapsed().as_secs_f64();
    // println!(
    //     "SEARCH TIME = {:.4}s  ({:.1} searches/sec)",
    //     search_time,
    //     n as f64 / search_time
    // );
    //
    // let k = t.leftmost_leaf();
}
// #[derive(Debug)]
// enum BTreeNode {
//     Internal {
//         parent: Option<Rc<RefCell<BTreeNode>>>,
//         keys: Vec<i32>,
//         children: Vec<Rc<RefCell<BTreeNode>>>,
//     },
//
//     Leaf {
//         parent: Option<Rc<RefCell<BTreeNode>>>,
//         data: Vec<Entry>,
//         next: Option<Rc<RefCell<BTreeNode>>>,
//     },
// }
// #[derive(Debug)]
// pub struct Entry {
//     key: i32,
//     data: String,
// }
//
// fn take(leaf: Rc<RefCell<BTreeNode>>) {
//     match &mut *leaf.borrow_mut() {
//         BTreeNode::Leaf {
//             parent: _,
//             data: vector,
//             next: _,
//         } => {
//             vector.push(Entry {
//                 key: 2,
//                 data: String::new(),
//             });
//         }
//         _ => unreachable!(),
//     }
// }
