use btree_rs::btree::{BTree, Entry};

#[test]
fn test_simple_insert_search() {
    let mut t = BTree::new(5);
    
    t.insert(Entry {
        key: 1,
        val: "one".to_string(),
    });
    
    assert!(t.search(&1).is_some());
    assert!(t.search(&2).is_none());
}
