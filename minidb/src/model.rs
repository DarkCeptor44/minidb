// Copyright (c) 2026, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

use redb::TableDefinition;
use serde::{Deserialize, Serialize};

/// A table model. A table model is a struct that implements the [`TableModel`] trait.
///
/// ## Example
///
/// ```rust,no_run
/// use minidb::TableModel;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Person {
///     id: String,
///     name: String,
///     age: u8,
/// }
///
/// impl TableModel for Person {
///     const TABLE: redb::TableDefinition<'_, &'static str, &[u8]> = redb::TableDefinition::new("people");
///
///     fn get_id(&self) -> &str {
///         &self.id
///     }
///
///     fn set_id(&mut self, id: String) {
///         self.id = id;
///     }
/// }
/// ```
pub trait TableModel: Serialize + for<'de> Deserialize<'de> {
    /// The table definition
    const TABLE: TableDefinition<'_, &'static str, &[u8]>;

    /// Returns the id of the table model
    fn get_id(&self) -> &str;

    /// Sets the id of the table model
    fn set_id(&mut self, id: String);
}
