use std::collections::HashSet;

use cuid2::is_slug;
use minidb::Id;

#[test]
fn test_id_collision() {
    const N: usize = 100000;

    let mut ids = HashSet::new();
    for _ in 0..N {
        let id = Id::<()>::generate();
        assert!(!ids.contains(&id));
        ids.insert(id);
    }
    assert_eq!(ids.len(), N);
}

#[test]
fn test_id_default() {
    let id = Id::<()>::generate();
    assert!(id.is_some());
    assert!(is_slug(id.value.unwrap()));
}

#[test]
fn test_id_new() {
    assert!(Id::<()>::new().is_none());
    assert!(Id::<()>::default().is_none());
}
