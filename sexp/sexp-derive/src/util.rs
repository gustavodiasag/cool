macro_rules! quote_block {
    ($($tt:tt)*) => {
        quote! {
            {
                $($tt)*
            }
        }
    };
}

pub(crate) use quote_block;
