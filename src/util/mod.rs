pub mod interner {
    use std::rc::Rc;

    use rustc_hash::FxHashMap;

    #[derive(Debug, PartialEq, Eq)]
    pub struct InternedIdx(u32);

    pub struct Interner {
        map: FxHashMap<Rc<str>, u32>,
        vec: Vec<Rc<str>>,
    }

    impl Interner {
        pub fn with_capacity(capacity: usize) -> Interner {
            Interner {
                map: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
                vec: Vec::with_capacity(capacity),
            }
        }

        pub fn intern(&mut self, name: &str) -> InternedIdx {
            if let Some(&idx) = self.map.get(name) {
                return InternedIdx(idx);
            }
            let name: Rc<str> = name.to_owned().into();
            let idx = self.map.len() as u32;

            self.map.insert(Rc::clone(&name), idx);
            self.vec.push(Rc::clone(&name));
            InternedIdx(idx)
        }

        pub fn lookup(&self, idx: &InternedIdx) -> &str {
            &self.vec[idx.0 as usize]
        }
    }
}

pub mod span {
    pub type Span = (usize, usize, usize, usize);

    #[derive(Debug, PartialEq)]
    pub struct Spanned<T> {
        pub inner: T,
        pub span: Span,
    }
}
