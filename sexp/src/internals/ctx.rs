use std::{cell::RefCell, fmt::Display, thread};

use quote::ToTokens;

#[derive(Default)]
pub struct Ctx {
    errors: RefCell<Option<Vec<syn::Error>>>,
}

impl Ctx {
    pub fn new() -> Ctx {
        Ctx {
            errors: RefCell::new(Some(Vec::new())),
        }
    }

    pub fn error_spanned_by<A: ToTokens, T: Display>(&self, obj: A, msg: T) {
        self.errors
            .borrow_mut()
            .as_mut()
            .unwrap()
            .push(syn::Error::new_spanned(obj.into_token_stream(), msg));
    }

    pub fn syn_error(&self, err: syn::Error) {
        self.errors.borrow_mut().as_mut().unwrap().push(err);
    }

    pub fn check(self) -> syn::Result<()> {
        let mut errors = self.errors.borrow_mut().take().unwrap().into_iter();

        let Some(mut combined) = errors.next() else {
            return Ok(());
        };

        for rest in errors {
            combined.combine(rest);
        }

        Err(combined)
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        if !thread::panicking() && self.errors.is_some() {
            panic!("forgot to check for errors");
        }
    }
}
