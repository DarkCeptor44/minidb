// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Dummy documentation block
//!
//! proc-macro crates don't have generated rustdocs

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations, missing_docs)]

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Ident, Lit, LitStr, Type, parse_macro_input};

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

#[derive(Debug, Default)]
struct MiniDBFieldAttributes {
    is_key: bool,
}

impl MiniDBFieldAttributes {
    fn from_attributes(attrs: &[Attribute]) -> Self {
        let mut field_attrs = Self::default();

        for attr in attrs {
            if attr.path().is_ident("key") {
                // #[key]
                field_attrs.is_key = true;
            }
        }

        field_attrs
    }
}

/// Derives `Table` for a struct
///
/// ## Attributes
///
/// ### Struct
///
/// * `#[minidb(name = "custom_name")]` - Sets a different name for the struct/table. Names get converted to `snake_case`
///
/// ### Field
///
/// * `#[key]` - Sets the field as a primary key
///
/// ## Example
///
/// ```rust,ignore
/// use minidb::Table;
///
/// #[derive(Table)]
/// #[minidb(name = "people")]
/// struct Person {
///     #[key]
///     id: String,
///     name: String,
///     age: u8,
/// }
/// ```
#[proc_macro_derive(Table, attributes(serde, minidb, key))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
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
        FoundCrate::Itself => quote!(minidb),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    };
    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
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

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "Struct field must have a name")
                .to_compile_error()
                .into();
        };

        let ty = &field.ty;
        let field_attrs = MiniDBFieldAttributes::from_attributes(&field.attrs);

        if field_attrs.is_key {
            num_keys_fields += 1;
            id_field_ident = Some(ident.clone());

            let is_id_type = is_id_type(ty);
            if !is_id_type {
                return Error::new_spanned(ty, "The #[key] field must be of type `String`.")
                    .to_compile_error()
                    .into();
            }
        }
    }

    if num_keys_fields != 1 {
        return Error::new_spanned(
            struct_name,
            "A struct deriving `Table` must have exactly one field marked with #[key].",
        )
        .to_compile_error()
        .into();
    }

    let Some(id_field_ident) = id_field_ident else {
        return Error::new_spanned(
            fields,
            "A struct deriving `Table` must have exactly one field marked with #[key].",
        )
        .to_compile_error()
        .into();
    };

    let table_model_impl = quote! {
        impl #impl_generics #crate_path::Table for #struct_name #ty_generics #where_clause {
            const TABLE: #crate_path::redb::TableDefinition<'_, &'static str, &[u8]> = #crate_path::redb::TableDefinition::new(#table_name);

            fn get_id(&self) -> &str {
                &self.#id_field_ident
            }

            fn set_id(&mut self, id: String) {
                self.#id_field_ident = id;
            }
        }
    };

    let out = quote! {
        #table_model_impl
    };

    // return Error::new_spanned(struct_name, out)
    //     .to_compile_error()
    //     .into();
    out.into()
}

fn is_id_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            last_segment.ident == "String"
        } else {
            false
        }
    } else {
        false
    }
}
