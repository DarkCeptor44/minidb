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

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Ident, Lit, LitStr, parse_macro_input};

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

/// Represents the attributes on a field
#[derive(Debug, Default)]
struct MiniDBFieldAttributes {
    is_key: bool,
    is_foreign_key: bool,
}

impl MiniDBFieldAttributes {
    fn from_attributes(attrs: &[Attribute]) -> Self {
        let mut field_attrs = Self::default();

        for attr in attrs {
            if attr.path().is_ident("key") {
                // #[key]
                field_attrs.is_key = true;
            } else if attr.path().is_ident("foreign_key") {
                // #[foreign_key]
                field_attrs.is_foreign_key = true;
            }
        }

        field_attrs
    }
}

#[proc_macro_derive(Table, attributes(serde, minidb, key, foreign_key))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let struct_generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();

    let struct_attrs = match MiniDBStructAttributes::from_attributes(&input.attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };
    let table_name_str = if let Some(custom_name) = struct_attrs.name {
        custom_name.to_snake_case()
    } else {
        struct_name.to_string().to_snake_case()
    };
    let table_name = Lit::Str(LitStr::new(&table_name_str, Span::call_site()));
    let Ok(found_crate) = crate_name("minidb") else {
        return Error::new_spanned(struct_name, "minidb crate not found in dependencies")
            .to_compile_error()
            .into();
    };
    let crate_path = match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    };
    let fields = match input.data {
        Data::Struct(s) => s.fields,
        Data::Enum(e) => {
            return Error::new_spanned(e.enum_token, "Table derive macro only supports structs")
                .to_compile_error()
                .into();
        }
        Data::Union(u) => {
            return Error::new_spanned(u.union_token, "Table derive macro only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut id_field_ident: Option<Ident> = None;
    let mut num_keys_fields = 0;
    let mut foreign_key_entries: Vec<TokenStream> = Vec::new();

    let out = quote! {};

    return Error::new_spanned(struct_name, out)
        .to_compile_error()
        .into();
    out.into()
}
