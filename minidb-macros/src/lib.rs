//! # minidb macros
//!
//! This crate provides macros for the `minidb` crate

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

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations, missing_docs)]

use syn::{Attribute, Error, Lit};

/// Represents the `minidb` attribute on a struct
#[derive(Debug, Default)]
struct MiniDBStructAttributes {
    name: Option<String>,
}

impl MiniDBStructAttributes {
    fn from_attributes(attrs: &[Attribute]) -> Result<Self, Error> {
        let mut struct_attrs = Self::default();

        for attr in attrs {
            if attr.path().is_ident("minidb") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let value: Lit = meta.value()?.parse()?;

                        if let Lit::Str(s) = value {
                            struct_attrs.name = Some(s.value());
                        } else {
                            return Err(meta.error("Expected string literal for `name` attribute"));
                        }
                    } else {
                        return Err(meta.error(
                            "Unknown minidb attribute on struct. Expected one of [`name`]",
                        ));
                    }

                    Ok(())
                })?;
            }
        }

        Ok(struct_attrs)
    }
}
