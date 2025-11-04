use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{Ident, Token, meta::ParseNestedMeta};

use crate::internals::{case::RenameRule, ctx::Ctx, name::Name, symbol::*};

pub(crate) struct Attr<'c, T> {
    ctx: &'c Ctx,
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(ctx: &'c Ctx, name: Symbol) -> Self {
        Attr {
            ctx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            let msg = format!("duplicate attribute `{}`", self.name);
            self.ctx.error_spanned_by(tokens, msg);
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    pub(crate) fn get(self) -> Option<T> {
        self.value
    }

    fn unwrap_or(self, other: T) -> T {
        match self.value {
            Some(v) => v,
            None => other,
        }
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(ctx: &'c Ctx, name: Symbol) -> Self {
        BoolAttr(Attr::none(ctx, name))
    }

    fn set_true<A: ToTokens>(&mut self, obj: A) {
        self.0.set(obj, ());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

pub struct Container {
    name: Name,
    rename_all: RenameRule,
    rename_all_fields: RenameRule,
    transparent: bool,
}

impl Container {
    /// Extract out the `#[sexp(...)]` attributes for an item.
    pub fn from_ast(ctx: &Ctx, item: &syn::DeriveInput) -> Container {
        let mut name = Attr::none(ctx, RENAME);
        let mut transparent = BoolAttr::none(ctx, TRANSPARENT);
        let mut rename_all = Attr::none(ctx, RENAME_ALL);
        let mut rename_all_fields = Attr::none(ctx, RENAME_ALL_FIELDS);

        for attr in &item.attrs {
            if attr.path() != SEXP {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    continue;
                }
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == RENAME {
                    let rename = get_rename(ctx, RENAME, &meta)?;
                    name.set_opt(&meta.path, rename.as_ref().map(Name::from));
                } else if meta.path == RENAME_ALL {
                    if let Some(rename) = get_rename(ctx, RENAME_ALL, &meta)? {
                        match RenameRule::from_str(&rename.value()) {
                            Ok(rule) => rename_all.set(&meta.path, rule),
                            Err(err) => ctx.error_spanned_by(rename, err),
                        }
                    }
                } else if meta.path == RENAME_ALL_FIELDS {
                    let rename = get_rename(ctx, RENAME_ALL_FIELDS, &meta)?;

                    match item.data {
                        syn::Data::Enum(_) => {
                            if let Some(rename) = rename {
                                match RenameRule::from_str(&rename.value()) {
                                    Ok(rule) => rename_all_fields.set(&meta.path, rule),
                                    Err(err) => ctx.error_spanned_by(rename, err),
                                }
                            }
                        }
                        _ => {
                            let msg = "#[sexp(rename_all_fields)] can only be used on enums";
                            ctx.syn_error(meta.error(msg));
                        }
                    }
                } else if meta.path == TRANSPARENT {
                    transparent.set_true(meta.path);
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown sexp container attribute `{}`", path))
                    );
                }
                Ok(())
            }) {
                ctx.syn_error(err);
            }
        }

        Container {
            name: name.unwrap_or(Name::from(&unraw(&item.ident))),
            rename_all: rename_all.get().unwrap_or_default(),
            rename_all_fields: rename_all_fields.get().unwrap_or_default(),
            transparent: transparent.get(),
        }
    }
    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_all_rule(&self) -> RenameRule {
        self.rename_all
    }

    pub fn rename_all_fields_rule(&self) -> RenameRule {
        self.rename_all_fields
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }
}

pub struct Variant {
    name: Name,
    rename_all: RenameRule,
    skip: bool,
}

impl Variant {
    pub fn from_ast(ctx: &Ctx, variant: &syn::Variant) -> Variant {
        let mut name = Attr::none(ctx, RENAME);
        let mut rename_all = Attr::none(ctx, RENAME_ALL);
        let mut skip = BoolAttr::none(ctx, SKIP);

        for attr in &variant.attrs {
            if attr.path() != SEXP {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    continue;
                }
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == RENAME {
                    let rename = get_rename(ctx, RENAME, &meta)?;
                    name.set_opt(&meta.path, rename.as_ref().map(Name::from));
                } else if meta.path == RENAME_ALL {
                    if let Some(rename) = get_rename(ctx, RENAME_ALL, &meta)? {
                        match RenameRule::from_str(&rename.value()) {
                            Ok(rule) => rename_all.set(&meta.path, rule),
                            Err(err) => ctx.error_spanned_by(rename, err),
                        }
                    }
                } else if meta.path == SKIP {
                    skip.set_true(&meta.path);
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown sexp variant attribute `{}`", path))
                    );
                }
                Ok(())
            }) {
                ctx.syn_error(err);
            }
        }

        Variant {
            name: name.unwrap_or(Name::from(&unraw(&variant.ident))),
            rename_all: rename_all.get().unwrap_or_default(),
            skip: skip.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_with(&mut self, rule: RenameRule) {
        self.name.value = rule.apply_to_variant(&self.name.value);
    }

    pub fn rename_all_rule(&self) -> RenameRule {
        self.rename_all
    }

    pub fn skip(&self) -> bool {
        self.skip
    }
}

pub struct Field {
    name: Name,
    skip: bool,
    flatten: bool,
    transparent: bool,
}

impl Field {
    /// Extract out the `#[sexp(...) attributes from a struct field]`.
    pub fn from_ast(ctx: &Ctx, idx: usize, field: &syn::Field) -> Field {
        let mut name = Attr::none(ctx, RENAME);
        let mut skip = BoolAttr::none(ctx, SKIP);
        let mut flatten = BoolAttr::none(ctx, FLATTEN);

        let ident = match &field.ident {
            Some(ident) => Name::from(&unraw(ident)),
            None => Name {
                value: idx.to_string(),
                span: Span::call_site(),
            },
        };

        for attr in &field.attrs {
            if attr.path() != SEXP {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    continue;
                }
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == RENAME {
                    let rename = get_rename(ctx, RENAME, &meta)?;
                    name.set_opt(&meta.path, rename.as_ref().map(Name::from));
                } else if meta.path == SKIP {
                    skip.set_true(&meta.path);
                } else if meta.path == FLATTEN {
                    flatten.set_true(&meta.path);
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown sexp field attribute `{}`", path)));
                }
                Ok(())
            }) {
                ctx.syn_error(err);
            }
        }

        Field {
            name: name.unwrap_or(ident),
            skip: skip.get(),
            flatten: flatten.get(),
            transparent: false,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_with(&mut self, rule: RenameRule) {
        self.name.value = rule.apply_to_field(&self.name.value);
    }

    pub fn skip(&self) -> bool {
        self.skip
    }

    pub fn flatten(&self) -> bool {
        self.flatten
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }
}

fn get_rename(
    ctx: &Ctx,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    let lookahead = meta.input.lookahead1();

    if !lookahead.peek(Token![=]) {
        return Err(lookahead.error());
    }
    let literal = {
        let expr: syn::Expr = meta.value()?.parse()?;
        let mut value = &expr;

        while let syn::Expr::Group(e) = value {
            value = &e.expr;
        }
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = value
        {
            let suffix = lit.suffix();
            if !suffix.is_empty() {
                ctx.error_spanned_by(
                    lit,
                    format!("unexpected suffix `{}` on string literal", suffix),
                );
            }
            Ok(Some(lit.clone()))
        } else {
            ctx.error_spanned_by(
                expr,
                format!("expected sexp {} attribute to be a string", attr_name),
            );
            Ok(None)
        }
    };
    literal
}

fn unraw(ident: &Ident) -> Ident {
    Ident::new(ident.to_string().trim_start_matches("r#"), ident.span())
}
