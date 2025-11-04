use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::LitStr;

pub struct Name {
    pub value: String,
    pub span: Span,
}

impl ToTokens for Name {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        LitStr::new(&self.value, self.span).to_tokens(tokens);
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Self {
        Name {
            value: ident.to_string(),
            span: ident.span(),
        }
    }
}

impl From<&LitStr> for Name {
    fn from(lit: &LitStr) -> Self {
        Name {
            value: lit.value(),
            span: lit.span(),
        }
    }
}
