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
use minidb_utils::{self as utils, ArgonParams};

const T: &[usize] = &[1, 4, 8, 16];

fn main() {
    divan::main();
}

#[divan::bench(args = [(1024, 2, 1), (19*1024, 3, 1), (64*1024, 3, 2)], threads = T)]
fn derive_key(b: Bencher, p: (u32, u32, u32)) {
    b.with_inputs(|| ArgonParams::new().m_cost(p.0).t_cost(p.1).p_cost(p.2))
        .bench_values(|p| {
            let key = utils::derive_key(black_box(p), black_box("password"), black_box("somesalt"))
                .expect("Failed to derive key");
            black_box(key);
        });
}

#[divan::bench(threads = T)]
fn generate_salt() {
    black_box(utils::generate_salt().expect("Failed to generate salt"));
}

#[divan::bench(args = [(1024, 2, 1), (19*1024, 3, 1), (64*1024, 3, 2)], threads = T)]
fn hash_password(b: Bencher, p: (u32, u32, u32)) {
    b.with_inputs(|| ArgonParams::new().m_cost(p.0).t_cost(p.1).p_cost(p.2))
        .bench_values(|p| {
            let hash =
                utils::hash_password(black_box(p), black_box("password"), black_box("somesalt"))
                    .expect("Failed to hash password");
            black_box(hash);
        });
}

#[divan::bench(args = [(1024, 2, 1), (19*1024, 3, 1), (64*1024, 3, 2)], threads = T)]
fn verify_password(b: Bencher, p: (u32, u32, u32)) {
    b.with_inputs(|| {
        let pass = "password";

        (
            pass,
            utils::hash_password(
                ArgonParams::new().m_cost(p.0).t_cost(p.1).p_cost(p.2),
                pass,
                "somesalt",
            )
            .expect("Failed to hash password"),
        )
    })
    .bench_values(|(pass, hash)| {
        black_box(
            utils::verify_password(black_box(pass), black_box(hash))
                .expect("Failed to verify password with hash"),
        );
    });
}