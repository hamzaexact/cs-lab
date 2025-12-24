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
    let mut t = BTree::new(5);
    let n = 1_000_000_usize;
    let mut rng = rand::thread_rng();
    let mut keys: Vec<i32> = (0..n as i32).collect();
    keys.shuffle(&mut rng);

    let keys = [10, 20, 30, 40, 50, 60, 70, 80, 90];
    insert_test(&mut t, &keys.into(), n);
    // search_test(&mut t, &keys, &mut rng, n);
    t.delete(10);
    t.delete(20);
    // t.delete(50);
    t.print_tree();
}

fn insert_test(t: &mut BTree, keys: &Vec<i32>, n: usize) {
    // println!("INSERTING {} keys...", n);

    for &k in keys {
        t.insert(Entry {
            key: k,
            data: k.to_string(),
        });
    }
}

fn search_test(t: &mut BTree, keys: &Vec<i32>, mut rng: &mut ThreadRng, n: usize) {
    println!("SEARCHING {} random keys...", n);
    let start_search = Instant::now();

    for _ in 0..n {
        let k = rng.gen_range(0..n as i32);
        let res = t.search(k);
        assert!(res.is_some());
    }
    let search_time = start_search.elapsed().as_secs_f64();
    println!(
        "SEARCH TIME = {:.4}s  ({:.1} searches/sec)",
        search_time,
        n as f64 / search_time
    );
}
