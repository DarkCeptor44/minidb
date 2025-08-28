use divan::{black_box, Bencher};
use minidb_utils::PathExt;
use tempfile::tempdir;

fn main() {
    divan::main();
}

#[divan::bench]
fn is_empty(b: Bencher) {
    b.with_inputs(|| {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().to_path_buf();

        (temp_dir, path)
    })
    .bench_values(|(_temp_dir, path)| {
        let r = black_box(path)
            .is_empty()
            .expect("Failed to check if path is empty");
        black_box(r);
    });
}
