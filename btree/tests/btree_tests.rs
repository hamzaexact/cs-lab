use btree_rs::btree::{BTree, Entry};
use rand::seq::SliceRandom;
use rand::Rng;
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
    v.shuffle(&mut rand::thread_rng());

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
                    if curr._cmp(&mut *node.borrow_mut(), btree_rs::btree::NodeCmpOrd::Less) {
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
                if let btree_rs::btree::BTreeNode::Leaf { data, next, .. } = &*node {
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
