//
//
//
//
//
//
mod btree;
use btree::btree::{BTree, Entry};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use std::time::{Duration, Instant};

fn main() {
    println!("B+ TREE COMPREHENSIVE BENCHMARK \n");

    // Test 1: Sequential operations (your current test)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 1: Sequential Operations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_sequential(1_000_000, 64);

    // Test 2: Random operations
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 2: Random Operations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_random(1_000_000, 64);

    // Test 3: Reverse sequential
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 3: Reverse Sequential");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_reverse(1_000_000, 64);

    // Test 4: Mixed workload (70% search, 20% insert, 10% delete)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 4: Mixed Workload (70/20/10)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_mixed_workload(1_000_000, 64);

    // Test 5: Different tree orders comparison
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 5: Order Comparison (100K keys)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_order_comparison(100_000);

    // Test 6: Range operations simulation
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 6: Range Query Simulation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_range_queries(1_000_000, 64);

    // Test 7: Skewed access pattern (80/20 rule)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 7: Skewed Access (80/20 pattern)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_skewed_access(1_000_000, 64);

    // Test 8: Alternating insert/delete
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 8: Alternating Insert/Delete");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_alternating(500_000, 64);

    // Test 9: Stress test with large dataset
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 9: Stress Test (5M keys)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_sequential(5_000_000, 64);
}

// Test 1: Sequential
fn test_sequential(n: usize, order: usize) {
    let mut tree = BTree::new(order);

    let start = Instant::now();
    for i in 0..n {
        tree.insert(Entry {
            key: i as i32,
            data: i.to_string(),
        });
    }
    let insert_time = start.elapsed();

    let start = Instant::now();
    for i in 0..n {
        let _ = tree.search(i as i32).unwrap();
    }
    let search_time = start.elapsed();

    let start = Instant::now();
    for i in 0..n {
        tree.delete(i as i32);
    }
    let delete_time = start.elapsed();

    print_results(n, insert_time, search_time, delete_time);
}

// Test 2: Random operations
fn test_random(n: usize, order: usize) {
    let mut tree = BTree::new(order);
    let mut rng = StdRng::seed_from_u64(42);

    let mut keys: Vec<i32> = (0..n as i32).collect();
    keys.shuffle(&mut rng);

    let start = Instant::now();
    for &key in &keys {
        tree.insert(Entry {
            key,
            data: key.to_string(),
        });
    }
    let insert_time = start.elapsed();

    keys.shuffle(&mut rng);
    let start = Instant::now();
    for &key in &keys {
        let _ = tree.search(key).unwrap();
    }
    let search_time = start.elapsed();

    keys.shuffle(&mut rng);
    let start = Instant::now();
    for &key in &keys {
        tree.delete(key);
    }
    let delete_time = start.elapsed();

    print_results(n, insert_time, search_time, delete_time);
}

// Test 3: Reverse sequential
fn test_reverse(n: usize, order: usize) {
    let mut tree = BTree::new(order);

    let start = Instant::now();
    for i in (0..n).rev() {
        tree.insert(Entry {
            key: i as i32,
            data: i.to_string(),
        });
    }
    let insert_time = start.elapsed();

    let start = Instant::now();
    for i in (0..n).rev() {
        let _ = tree.search(i as i32).unwrap();
    }
    let search_time = start.elapsed();

    let start = Instant::now();
    for i in (0..n).rev() {
        tree.delete(i as i32);
    }
    let delete_time = start.elapsed();

    print_results(n, insert_time, search_time, delete_time);
}

// Test 4: Mixed workload
fn test_mixed_workload(n: usize, order: usize) {
    let mut tree = BTree::new(order);
    let mut rng = StdRng::seed_from_u64(42);

    // Pre-populate tree
    for i in 0..n {
        tree.insert(Entry {
            key: i as i32,
            data: i.to_string(),
        });
    }

    let ops = n * 2; // 2x operations
    let start = Instant::now();

    for _ in 0..ops {
        let op = rng.random_range(0..100);
        let key = rng.random_range(0..n as i32);

        if op < 70 {
            // 70% search
            let _ = tree.search(key);
        } else if op < 90 {
            // 20% insert
            tree.insert(Entry {
                key: key + n as i32,
                data: key.to_string(),
            });
        } else {
            // 10% delete
            tree.delete(key);
        }
    }

    let total_time = start.elapsed();
    println!("Total operations: {}", ops);
    println!("Total time: {:?}", total_time);
    println!(
        "Ops/sec: {:.2} M",
        (ops as f64 / total_time.as_secs_f64()) / 1_000_000.0
    );
}

// Test 5: Order comparison
fn test_order_comparison(n: usize) {
    let orders = vec![3, 5, 16, 32, 64, 128, 256];

    println!(
        "\n{:<8} {:<12} {:<12} {:<12}",
        "Order", "Insert(M/s)", "Search(M/s)", "Delete(M/s)"
    );
    println!("{}", "─".repeat(50));

    for order in orders {
        let mut tree = BTree::new(order);

        let start = Instant::now();
        for i in 0..n {
            tree.insert(Entry {
                key: i as i32,
                data: i.to_string(),
            });
        }
        let insert_rate = n as f64 / start.elapsed().as_secs_f64() / 1_000_000.0;

        let start = Instant::now();
        for i in 0..n {
            let _ = tree.search(i as i32);
        }
        let search_rate = n as f64 / start.elapsed().as_secs_f64() / 1_000_000.0;

        let start = Instant::now();
        for i in 0..n {
            tree.delete(i as i32);
        }
        let delete_rate = n as f64 / start.elapsed().as_secs_f64() / 1_000_000.0;

        println!(
            "{:<8} {:<12.2} {:<12.2} {:<12.2}",
            order, insert_rate, search_rate, delete_rate
        );
    }
}

// Test 6: Range query simulation
fn test_range_queries(n: usize, order: usize) {
    let mut tree = BTree::new(order);

    // Insert
    for i in 0..n {
        tree.insert(Entry {
            key: i as i32,
            data: i.to_string(),
        });
    }

    // Simulate 1000 range queries of size 1000 each
    let ranges = 1000;
    let range_size = 1000;
    let mut rng = StdRng::seed_from_u64(42);

    let start = Instant::now();
    for _ in 0..ranges {
        let start_key = rng.random_range(0..(n - range_size) as i32);
        for key in start_key..(start_key + range_size as i32) {
            let _ = tree.search(key);
        }
    }
    let duration = start.elapsed();

    let total_ops = ranges * range_size;
    println!("Range queries: {}", ranges);
    println!("Keys per range: {}", range_size);
    println!("Total keys scanned: {}", total_ops);
    println!("Time: {:?}", duration);
    println!(
        "Keys/sec: {:.2} M",
        (total_ops as f64 / duration.as_secs_f64()) / 1_000_000.0
    );
}

// Test 7: Skewed access (80/20 rule)
fn test_skewed_access(n: usize, order: usize) {
    let mut tree = BTree::new(order);
    let mut rng = StdRng::seed_from_u64(42);

    // Insert all keys
    for i in 0..n {
        tree.insert(Entry {
            key: i as i32,
            data: i.to_string(),
        });
    }

    // 80% of accesses go to 20% of keys
    let hot_keys = n / 5; // 20% of keys
    let ops = n * 2;

    let start = Instant::now();
    for _ in 0..ops {
        let key = if rng.random_range(0..100) < 80 {
            // 80% - access hot keys
            rng.random_range(0..hot_keys as i32)
        } else {
            // 20% - access all keys
            rng.random_range(0..n as i32)
        };
        let _ = tree.search(key);
    }
    let duration = start.elapsed();

    println!("Hot keys (20%): {}", hot_keys);
    println!("Total searches: {}", ops);
    println!("Time: {:?}", duration);
    println!(
        "Ops/sec: {:.2} M",
        (ops as f64 / duration.as_secs_f64()) / 1_000_000.0
    );
}

// Test 8: Alternating insert/delete
fn test_alternating(n: usize, order: usize) {
    let mut tree = BTree::new(order);

    let start = Instant::now();

    for i in 0..n {
        tree.insert(Entry {
            key: i as i32,
            data: i.to_string(),
        });

        if i > 0 && i % 2 == 0 {
            tree.delete((i - 1) as i32);
        }
    }

    let duration = start.elapsed();
    let total_ops = n + (n / 2); // inserts + deletes

    println!("Inserts: {}", n);
    println!("Deletes: {}", n / 2);
    println!("Total ops: {}", total_ops);
    println!("Time: {:?}", duration);
    println!(
        "Ops/sec: {:.2} M",
        (total_ops as f64 / duration.as_secs_f64()) / 1_000_000.0
    );
}

// Helper function
fn print_results(n: usize, insert: Duration, search: Duration, delete: Duration) {
    println!("Keys: {}", n);
    println!(
        "Insert: {:?} ({:.2} M ops/s)",
        insert,
        n as f64 / insert.as_secs_f64() / 1_000_000.0
    );
    println!(
        "Search: {:?} ({:.2} M ops/s)",
        search,
        n as f64 / search.as_secs_f64() / 1_000_000.0
    );
    println!(
        "Delete: {:?} ({:.2} M ops/s)",
        delete,
        n as f64 / delete.as_secs_f64() / 1_000_000.0
    );
}
