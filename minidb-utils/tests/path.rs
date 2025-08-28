use std::{fs::File, io::Write};

use minidb_utils::PathExt;
use tempfile::tempdir;

#[test]
fn test_pathext_is_empty() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path();
    assert!(
        temp_path
            .is_empty()
            .expect("Failed to check if path is empty"),
        "Path is not empty"
    );

    let mut file = File::create(temp_path.join("test")).expect("Failed to create file");
    file.write_all(b"Hello world")
        .expect("Failed to write to file");
    assert!(
        !temp_path
            .is_empty()
            .expect("Failed to check if path is empty"),
        "Path is empty when it should not be"
    );
}

// TODO add more tests
