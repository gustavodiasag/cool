use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{Ident, Token, meta::ParseNestedMeta};

use crate::internals::{case::RenameRule, name::Name, symbol::*};

pub(crate) struct Attr<T> {
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<T> Attr<T> {
    fn none(name: Symbol) -> Self {
        Attr {
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            let msg = format!("duplicate attribute `{}`", self.name);
            // TODO: error handling
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

    fn get_or(self, other: T) -> T {
        match self.value {
            Some(v) => v,
            None => other,
        }
    }

    fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}

struct BoolAttr(Attr<()>);

impl BoolAttr {
    fn none(name: Symbol) -> BoolAttr {
        BoolAttr(Attr::none(name))
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
    transparent: bool,
    rename_all: RenameRule,
    rename_all_fields: RenameRule,
}

impl Container {
    /// Extract out the `#[sexp(...)]` attributes for an item.
    pub fn from_ast(item: &syn::DeriveInput) -> Container {
        let mut name = Attr::none(RENAME);
        let mut transparent = BoolAttr::none(TRANSPARENT);
        let mut rename_all = Attr::none(RENAME_ALL);
        let mut rename_all_fields = Attr::none(RENAME_ALL_FIELDS);

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
                    let rename = get_rename(RENAME, &meta)?;
                    name.set_opt(&meta.path, rename.as_ref().map(Name::from));
                } else if meta.path == RENAME_ALL {
                    if let Some(rename) = get_rename(RENAME_ALL, &meta)? {
                        match RenameRule::from_str(&rename.value()) {
                            Ok(rule) => rename_all.set(&meta.path, rule),
                            Err(err) => todo!(),
                        }
                    }
                } else if meta.path == RENAME_ALL_FIELDS {
                    let rename = get_rename(RENAME_ALL_FIELDS, &meta)?;

                    match item.data {
                        syn::Data::Enum(_) => {
                            if let Some(rename) = rename {
                                match RenameRule::from_str(&rename.value()) {
                                    Ok(rule) => rename_all_fields.set(&meta.path, rule),
                                    Err(err) => todo!(),
                                }
                            }
                        }
                        _ => {
                            todo!()
                        }
                    }
                } else if meta.path == TRANSPARENT {
                    transparent.set_true(meta.path);
                }
                Ok(())
            }) {
                todo!()
            }
        }

        Container {
            name: name.get_or(Name::from(&unraw(&item.ident))),
            transparent: transparent.get(),
            rename_all: rename_all.get().unwrap_or_default(),
            rename_all_fields: rename_all_fields.get().unwrap_or_default(),
        }
    }
}

pub struct Variant {
    name: Name,
    rename_all: RenameRule,
    skip: bool,
}

impl Variant {
    pub fn from_ast(variant: &syn::Variant) -> Variant {
        let mut name = Attr::none(RENAME);
        let mut rename_all = Attr::none(RENAME_ALL);
        let mut skip = BoolAttr::none(SKIP);

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
                    let rename = get_rename(RENAME, &meta)?;
                    name.set_opt(&meta.path, rename.as_ref().map(Name::from));
                } else if meta.path == RENAME_ALL {
                    if let Some(rename) = get_rename(RENAME_ALL, &meta)? {
                        match RenameRule::from_str(&rename.value()) {
                            Ok(rule) => rename_all.set(&meta.path, rule),
                            Err(err) => todo!(),
                        }
                    }
                } else if meta.path == SKIP {
                    skip.set_true(&meta.path);
                }
                Ok(())
            }) {
                todo!()
            }
        }

        Variant {
            name: name.get_or(Name::from(&unraw(&variant.ident))),
            rename_all: rename_all.get().unwrap_or_default(),
            skip: skip.get(),
        }
    }
}

pub struct Field {
    name: Name,
    skip: bool,
    flatten: bool,
}

impl Field {
    /// Extract out the `#[sexp(...) attributes from a struct field]`.
    pub fn from_ast(idx: usize, field: &syn::Field) -> Field {
        let mut name = Attr::none(RENAME);
        let mut skip = BoolAttr::none(SKIP);
        let mut flatten = BoolAttr::none(FLATTEN);

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
                    let rename = get_rename(RENAME, &meta)?;
                    name.set_opt(&meta.path, rename.as_ref().map(Name::from));
                } else if meta.path == SKIP {
                    skip.set_true(&meta.path);
                } else if meta.path == FLATTEN {
                    flatten.set_true(&meta.path);
                }
                Ok(())
            }) {
                todo!()
            }
        }

        Field {
            name: name.get_or(ident),
            skip: skip.get(),
            flatten: flatten.get(),
        }
    }
}

fn get_rename(attr_name: Symbol, meta: &ParseNestedMeta) -> syn::Result<Option<syn::LitStr>> {
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
            Ok(Some(lit.clone()))
        } else {
            Ok(None)
        }
    };
    literal
}

fn unraw(ident: &Ident) -> Ident {
    Ident::new(ident.to_string().trim_start_matches("r#"), ident.span())
}
