use std::fmt::{self};

use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const FLATTEN: Symbol = Symbol("flatten");
pub const RENAME: Symbol = Symbol("rename");
pub const RENAME_ALL: Symbol = Symbol("rename_all");
pub const RENAME_ALL_FIELDS: Symbol = Symbol("rename_all_fields");
pub const SEXP: Symbol = Symbol("sexp");
pub const SKIP: Symbol = Symbol("skip");
pub const TRANSPARENT: Symbol = Symbol("transparent");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl PartialEq<Symbol> for &Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl PartialEq<Symbol> for &Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
