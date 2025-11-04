use syn::{Token, punctuated::Punctuated};

use crate::internals::{attr, ctx::Ctx};

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Container,
    pub data: Data<'a>,
    pub original: &'a syn::DeriveInput,
}

pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Variant,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
    pub original: &'a syn::Variant,
}

pub struct Field<'a> {
    pub member: syn::Member,
    pub attrs: attr::Field,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}

#[derive(Copy, Clone)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    NewType,
    /// No fields.
    Unit,
}

impl<'a> Container<'a> {
    pub fn from_ast(ctx: &Ctx, item: &'a syn::DeriveInput) -> Option<Container<'a>> {
        let attrs = attr::Container::from_ast(ctx, item);

        let mut data = match &item.data {
            syn::Data::Enum(data) => Data::Enum(enum_from_ast(ctx, &data.variants)),
            syn::Data::Struct(data) => {
                let (style, fields) = struct_from_ast(ctx, &data.fields);
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                ctx.error_spanned_by(item, "Sexp does not support derive for unions");
                return None;
            }
        };

        match &mut data {
            Data::Enum(variants) => variants.iter_mut().for_each(|variant| {
                variant.attrs.rename_with(attrs.rename_all_rule());
                variant.fields.iter_mut().for_each(|field| {
                    field.attrs.rename_with(
                        variant
                            .attrs
                            .rename_all_rule()
                            .or(attrs.rename_all_fields_rule()),
                    )
                });
            }),
            Data::Struct(_, fields) => {
                fields
                    .iter_mut()
                    .for_each(|field| field.attrs.rename_with(attrs.rename_all_rule()));
            }
        }

        Some(Container {
            ident: item.ident.clone(),
            attrs,
            data,
            original: item,
        })
    }
}

fn enum_from_ast<'a>(
    ctx: &Ctx,
    variants: &'a Punctuated<syn::Variant, Token![,]>,
) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(|variant| {
            let attrs = attr::Variant::from_ast(ctx, variant);
            let (style, fields) = struct_from_ast(ctx, &variant.fields);

            Variant {
                ident: variant.ident.clone(),
                attrs,
                style,
                fields,
                original: variant,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(ctx: &Ctx, fields: &'a syn::Fields) -> (Style, Vec<Field<'a>>) {
    match fields {
        syn::Fields::Named(fields) => (Style::Struct, fields_from_ast(ctx, &fields.named)),
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            (Style::NewType, fields_from_ast(ctx, &fields.unnamed))
        }
        syn::Fields::Unnamed(fields) => (Style::Tuple, fields_from_ast(ctx, &fields.unnamed)),
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(ctx: &Ctx, fields: &'a Punctuated<syn::Field, Token![,]>) -> Vec<Field<'a>> {
    fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let member = match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                _ => syn::Member::Unnamed(idx.into()),
            };

            Field {
                member,
                attrs: attr::Field::from_ast(ctx, idx, field),
                ty: &field.ty,
                original: field,
            }
        })
        .collect()
}
