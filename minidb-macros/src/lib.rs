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
use syn::{
    Attribute, Data, DeriveInput, Error, GenericArgument, Ident, Lit, LitStr, PathArguments, Type,
    parse_macro_input,
};

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

/// Derives `AsTable` for a struct
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
/// * `#[foreign_key]` - Sets the field as a foreign key to the referenced table's primary key, for example:
///
/// ```rust,ignore
/// #[foreign_key]
/// customer_id: Id<Person>, // references the primary key of the Person table
/// ```
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
///     id: Id<Self>,
///     name: String,
///     age: u8,
/// }
/// ```
#[proc_macro_derive(Table, attributes(serde, minidb, key, foreign_key))]
#[allow(clippy::too_many_lines)]
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
        FoundCrate::Itself => quote!(crate),
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
    let mut foreign_key_entries = Vec::new();

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
                return Error::new_spanned(ty, "The #[key] field must be of type `Id<Self>`.")
                    .to_compile_error()
                    .into();
            }
        }

        if field_attrs.is_foreign_key {
            let ref_table = match get_ref_table(ty) {
                Ok(t) => t,
                Err(e) => return e.to_compile_error().into(),
            };

            foreign_key_entries.push(quote! {
                (stringify!(#ident), #ref_table, Box::new(|s: &Self| s.#ident.value.as_deref()))
            });
        }
    }

    if num_keys_fields != 1 {
        return Error::new_spanned(
            struct_name,
            "A struct deriving `Table` must have exactly one field marked with `#[key]`.",
        )
        .to_compile_error()
        .into();
    }

    let Some(id_field_ident) = id_field_ident else {
        return Error::new_spanned(
            fields,
            "A struct deriving `Table` must have exactly one field marked with `#[key]`.",
        )
        .to_compile_error()
        .into();
    };

    let as_table_impl = quote! {
        impl #crate_path::AsTable for #struct_name #impl_generics #ty_generics #where_clause {
            fn name() -> &'static str {
                #table_name
            }

            fn get_id(&self) -> &#crate_path::Id<Self> {
                &self.#id_field_ident
            }

            fn set_id(&mut self, id: #crate_path::Id<Self>) {
                self.#id_field_ident = id;
            }

            fn get_foreign_keys() -> Vec<(&'static str, &'static str, Box<dyn Fn(&Self) -> Option<&str> + Send + Sync>)> {
                vec![
                    #(#foreign_key_entries),*
                ]
            }
        }
    };

    let out = quote! {
        #as_table_impl
    };

    // return Error::new_spanned(struct_name, out)
    //     .to_compile_error()
    //     .into();
    out.into()
}

fn is_id_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if last_segment.ident == "Id" {
                // checks if it has generic arguments
                if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    // check if it has only 1 generic argument
                    args.args.len() == 1
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

fn get_ref_table(ty: &Type) -> Result<String, Error> {
    match ty {
        Type::Path(type_path) => match type_path.path.segments.last() {
            Some(last_segment) => {
                if last_segment.ident == "Id" {
                    match &last_segment.arguments {
                        PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
                            match &args.args[0] {
                                GenericArgument::Type(Type::Path(type_path)) => {
                                    match type_path.path.segments.last() {
                                        Some(inner_last_segment) => {
                                            Ok(inner_last_segment.ident.to_string())
                                        }
                                        _ => Err(Error::new_spanned(
                                            ty,
                                            "Foreign key field must be of type `Id<Table>`.",
                                        )),
                                    }
                                }
                                _ => Err(Error::new_spanned(
                                    ty,
                                    "Foreign key field must be of type `Id<Table>`.",
                                )),
                            }
                        }
                        _ => Err(Error::new_spanned(
                            ty,
                            "Foreign key field must be of type `Id<Table>`.",
                        )),
                    }
                } else {
                    Err(Error::new_spanned(
                        ty,
                        "Foreign key field must be of type `Id<Table>`.",
                    ))
                }
            }
            _ => Err(Error::new_spanned(
                ty,
                "Foreign key field must be of type `Id<Table>`.",
            )),
        },
        _ => Err(Error::new_spanned(
            ty,
            "Foreign key field must be of type `Id<Table>`.",
        )),
    }
}
