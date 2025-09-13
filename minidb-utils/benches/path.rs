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

use divan::{Bencher, black_box};
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
