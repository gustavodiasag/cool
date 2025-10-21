use proc_macro2::TokenStream;
use syn::*;

mod internals;

#[proc_macro_derive(Sexp, attributes(sexp))]
pub fn derive_sexp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = &input.ident;

    let impl_block = match &input.data {
        Data::Struct(data) => impl_struct_sexp(name, &data.fields),
        Data::Enum(data) => impl_enum_sexp(name, data),
        _ => unimplemented!(),
    };

    impl_block.into()
}

fn impl_struct_sexp(name: &Ident, fields: &Fields) -> TokenStream {
    todo!()
}

fn impl_enum_sexp(name: &Ident, data: &DataEnum) -> TokenStream {
    todo!()
}
