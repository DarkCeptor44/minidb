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

#[test]
fn test_pathext_is_empty_for_file() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let file_path = temp_dir.path().join("file.txt");
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"Hello world")
        .expect("Failed to write to file");

    assert!(
        !file_path
            .is_empty()
            .expect("Failed to check if path is empty"),
        "File path should not be considered an empty directory"
    );
}

#[test]
fn test_pathext_is_empty_for_non_existent_path() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let non_existent_path = temp_dir.path().join("non_existent");

    assert!(
        !non_existent_path
            .is_empty()
            .expect("is_empty should not fail for non-existent path"),
        "Non-existent path should not be considered an empty directory"
    );
}