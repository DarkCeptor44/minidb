use minidb_utils::read_from_file;
use tempfile::NamedTempFile;

#[test]
fn test_read_from_file() {
    const CONTENT: &str = "Hello world";

    let file = NamedTempFile::new().expect("Failed to create temporary file");
    let path = file.path();

    std::fs::write(path, CONTENT).expect("Failed to write to file");

    let s = read_from_file(path).expect("Failed to read file");
    assert_eq!(s, CONTENT);
}
