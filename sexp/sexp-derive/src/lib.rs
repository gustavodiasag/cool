use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned};

use crate::{
    internals::{
        Ctx,
        ast::{Container, Data, Field, Style, Variant},
        attr,
        name::Name,
    },
    util::quote_block,
};

mod internals;
mod util;

#[proc_macro_derive(Sexp, attributes(sexp))]
pub fn derive_sexp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as syn::DeriveInput);

    expand_derive_sexp(&mut input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_derive_sexp(input: &mut syn::DeriveInput) -> syn::Result<TokenStream> {
    let ctx = Ctx::new();

    let Some(container) = Container::from_ast(&ctx, input) else {
        return Err(ctx.check().unwrap_err());
    };
    ctx.check()?;

    let expanded = {
        let ident = &container.ident;
        let this = syn::Path::from(container.ident.clone());
        let body = get_body(&container, &this);

        quote! {
            impl sexp_core::ToSexp for #ident {
                fn to_sexp<S>(&self, s: &mut S) -> sexp_core::Result<()>
                where
                    S: sexp_core::SexpSerializer,
                {
                    #body
                }
            }
        }
    };
    Ok(expanded)
}

fn get_body(container: &Container, this: &syn::Path) -> TokenStream {
    if container.attrs.transparent() {
        get_transparent(container)
    } else {
        match &container.data {
            Data::Enum(variants) => get_enum(this, variants, &container.attrs),
            Data::Struct(Style::Struct, fields) => get_struct(fields, &container.attrs),
            Data::Struct(Style::Tuple, fields) => get_tuple_struct(fields, &container.attrs),
            Data::Struct(Style::NewType, fields) => {
                get_newtype_struct(&fields[0], &container.attrs)
            }
            Data::Struct(Style::Unit, _) => get_unit_struct(&container.attrs),
        }
    }
}

fn get_transparent(container: &Container) -> TokenStream {
    let fields = match &container.data {
        Data::Struct(_, fields) => fields,
        Data::Enum(_) => unreachable!(),
    };
    let transparent = fields
        .iter()
        .find(|field| field.attrs.transparent())
        .unwrap();
    let path = {
        let span = transparent.original.span();
        quote_spanned!(span=> sexp_core::ToSexp::to_sexp)
    };
    let member = &transparent.member;

    quote_block! {
        #path(&self.#member)
    }
}

fn get_enum(this: &syn::Path, variants: &[Variant], attrs: &attr::Container) -> TokenStream {
    assert!(variants.len() as u64 <= u64::from(u32::MAX));

    let arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| get_variant(this, variant, idx as u32, attrs))
        .collect();

    quote! {
        match *self {
            #(#arms)*
        }
    }
}

fn get_variant(
    this: &syn::Path,
    variant: &Variant,
    idx: u32,
    attrs: &attr::Container,
) -> TokenStream {
    let variant_ident = &variant.ident;

    if variant.attrs.skip() {
        let err = {
            let msg = format!(
                "the enum variant {}::{} cannot be represented",
                type_ident(this),
                variant_ident
            );
            quote!(Err(std::io::Error::other(#msg)))
        };
        let fields_pat = match variant.style {
            Style::Unit => quote!(),
            Style::NewType | Style::Tuple => quote!((..)),
            Style::Struct => quote!({ .. }),
        };
        quote! {
            #this::#variant_ident #fields_pat => #err,
        }
    } else {
        let case = match variant.style {
            Style::Unit => {
                quote! {
                    #this::#variant_ident
                }
            }
            Style::NewType => {
                quote! {
                    #this::#variant_ident(ref field)
                }
            }
            Style::Tuple => {
                let fields = (0..variant.fields.len())
                    .map(|i| Ident::new(&format!("field{}", i), Span::call_site()));
                quote! {
                    #this::#variant_ident(#(ref #fields),*)
                }
            }
            Style::Struct => {
                let members = variant.fields.iter().map(|field| &field.member);
                quote! {
                    #this::#variant_ident { #(ref #members),* }
                }
            }
        };
        let body = {
            let type_name = attrs.name();
            let variant_name = variant.attrs.name();

            match filtered_variant_style(variant) {
                Style::Unit => {
                    quote_block!(s.serialize_unit_variant(#type_name, #idx, #variant_name))
                }
                Style::NewType => {
                    quote_block!(s.serialize_newtype_variant(#type_name, #idx, #variant_name, field))
                }
                Style::Tuple => get_tuple_variant(type_name, idx, variant_name, &variant.fields),
                Style::Struct => get_struct_variant(type_name, idx, variant_name, &variant.fields),
            }
        };
        quote! {
            #case => #body
        }
    }
}

fn get_tuple_variant(
    type_name: &Name,
    idx: u32,
    variant_name: &Name,
    fields: &[Field],
) -> TokenStream {
    let stmts = get_tuple_fields(fields, true);
    let len = {
        let count = fields.iter().filter(|field| !field.attrs.skip()).count();
        quote!(#count)
    };
    quote_block! {
        s.serialize_tuple_variant(#type_name, #idx, #variant_name, #len)?;
        #(#stmts)*
        s.end_cell()
    }
}

fn get_struct_variant(
    type_name: &Name,
    idx: u32,
    variant_name: &Name,
    fields: &[Field],
) -> TokenStream {
    let stmts = get_struct_fields(fields, true);

    quote_block! {
        s.serialize_struct_variant(#variant_name)?;
        #(#stmts)*
        s.end_cell()
    }
}

fn get_struct(fields: &[Field], attrs: &attr::Container) -> TokenStream {
    let type_name = attrs.name();
    let stmts = get_struct_fields(fields, false);

    quote_block! {
        s.serialize_struct(#type_name)?;
        #(#stmts)*
        s.end_cell()
    }
}

fn get_tuple_struct(fields: &[Field], attrs: &attr::Container) -> TokenStream {
    let type_name = attrs.name();
    let stmts = get_tuple_fields(fields, false);

    quote_block! {
        s.serialize_tuple_struct(#type_name)?;
        #(#stmts)*
        s.end_cell()
    }
}

fn get_newtype_struct(field: &Field, attrs: &attr::Container) -> TokenStream {
    let type_name = attrs.name();

    let field_expr = {
        let member = syn::Member::Unnamed(syn::Index {
            index: 0,
            span: Span::call_site(),
        });
        quote!(&self.#member)
    };
    let span = field.original.span();

    quote_spanned!(span=> s.serialize_newtype_struct(#type_name, #field_expr))
}

fn get_unit_struct(attrs: &attr::Container) -> TokenStream {
    let type_name = attrs.name();

    quote! {
        s.serialize_unit_struct(#type_name)
    }
}

fn get_tuple_fields(fields: &[Field], is_enum: bool) -> Vec<TokenStream> {
    fields
        .iter()
        .enumerate()
        .filter(|(_, field)| !field.attrs.skip())
        .map(|(i, field)| {
            let field_expr = if is_enum {
                let id = Ident::new(&format!("field{}", i), Span::call_site());
                quote!(#id)
            } else {
                let member = syn::Member::Unnamed(syn::Index {
                    index: i as u32,
                    span: Span::call_site(),
                });
                quote!(&self.#member)
            };
            let span = field.original.span();

            quote_spanned!(span=> s.serialize_tuple_field(#field_expr)?;)
        })
        .collect()
}

fn get_struct_fields(fields: &[Field], is_enum: bool) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|&field| !field.attrs.skip())
        .map(|field| {
            let member = &field.member;

            let field_expr = if is_enum {
                quote!(#member)
            } else {
                quote!(&self.#member)
            };
            let key = field.attrs.name();
            let span = field.original.span();

            quote_spanned!(span=> s.serialize_struct_field(#key, #field_expr)?;)
        })
        .collect()
}

fn type_ident(path: &syn::Path) -> String {
    path.segments.last().unwrap().ident.to_string()
}

fn filtered_variant_style(variant: &Variant) -> Style {
    match variant.style {
        Style::NewType if variant.fields[0].attrs.skip() => Style::Unit,
        other => other,
    }
}
