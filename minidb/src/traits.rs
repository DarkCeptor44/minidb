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

use crate::Id;

type ForeignKeyTuple<S> = (
    &'static str,
    &'static str,
    Box<dyn Fn(&S) -> Option<&str> + Send + Sync>,
);

/// A trait for defining a table
pub trait AsTable: Sized {
    /// The name of the table in `snake_case`
    fn name() -> &'static str;

    /// The primary key of the table
    fn get_id(&self) -> &Id<Self>;

    /// Sets the primary key of the table
    fn set_id(&mut self, id: Id<Self>);

    /// The foreign keys of the table
    ///
    /// ## Returns
    ///
    /// A vector of tuples in the format `(field_name, referenced_table, get_foreign_key)`
    fn get_foreign_keys() -> Vec<ForeignKeyTuple<Self>>;
}
