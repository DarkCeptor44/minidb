// Copyright (c) 2025, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

use divan::{black_box, Bencher};
use minidb_utils as utils;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

const N: u64 = 1024;

fn main() {
    divan::main();
}

#[divan::bench]
fn read_from_file(b: Bencher) {
    b.with_inputs(|| {
        let content = padding(N);
        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let path = file.path().to_path_buf();

        std::fs::write(&path, content).expect("Failed to write to file");

        (file, path)
    })
    .bench_values(|(_file, path)| {
        let s = utils::read_from_file(black_box(path)).expect("Failed to read file");
        black_box(s);
    });
}

#[divan::bench]
fn read_from_file_async(b: Bencher) {
    let rt = Runtime::new().expect("Failed to create runtime");

    b.with_inputs(|| {
        let content = padding(N);
        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let path = file.path().to_path_buf();

        std::fs::write(&path, content).expect("Failed to write to file");

        (file, path)
    })
    .bench_values(|(_file, path)| {
        let s = rt
            .block_on(utils::read_from_file_async(black_box(path)))
            .expect("Failed to read file");
        black_box(s)
    });
}

fn padding(bytes: u64) -> Vec<u8> {
    const CONTENT: &str = "content8adsaasdadasdadlklaklskdklaslkd";

    CONTENT
        .as_bytes()
        .iter()
        .cycle()
        .take(bytes as usize)
        .cloned()
        .collect()
}
