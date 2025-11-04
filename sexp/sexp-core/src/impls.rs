use std::collections::VecDeque;

use crate::{Result, SexpSerializer, ToSexp};

macro_rules! primitive_impls {
    ($($ty:ty, $func:ident),+) => {
        $(
            impl ToSexp for $ty {
                fn to_sexp<S>(&self, serializer: &mut S) -> Result<()>
                where
                    S: SexpSerializer,
                {
                    serializer.$func(*self)
                }
            }
        )+
    };
}

primitive_impls! {
    bool, serialize_bool,
    u8, serialize_u8,
    u16, serialize_u16,
    u32, serialize_u32,
    u64, serialize_u64,
    u128, serialize_u128,
    i8, serialize_i8,
    i16, serialize_i16,
    i32, serialize_i32,
    i64, serialize_i64,
    i128, serialize_i128,
    f32, serialize_f32,
    f64, serialize_f64,
    char, serialize_char
}

impl ToSexp for str {
    fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()> {
        serializer.serialize_str(self)
    }
}

impl ToSexp for String {
    fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()> {
        serializer.serialize_str(self)
    }
}

impl<T> ToSexp for Option<T>
where
    T: ToSexp,
{
    fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()> {
        match *self {
            Some(ref value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

impl<T> ToSexp for [T]
where
    T: ToSexp,
{
    fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()> {
        self.into_iter()
            .try_for_each(|item| item.to_sexp(serializer))
    }
}

macro_rules! seq_impls {
    ($($ty:ident <T>),+) => {
        $(
            impl<T> ToSexp for $ty<T>
            where
                T: ToSexp,
            {
                fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()> {
                    self.into_iter()
                        .try_for_each(|item| item.to_sexp(serializer))
                }
            }
        )+
    };
}

seq_impls! {
    Vec<T>,
    VecDeque<T>
}

macro_rules! deref_impl {
    (
        <$($desc:tt)+
    ) => {
        impl <$($desc)+ {
            fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()> {
                (**self).to_sexp(serializer)
            }
        }
    };
}

deref_impl! {
    <'a, T> ToSexp for &'a T where T: ToSexp
}

deref_impl! {
    <'a, T> ToSexp for &'a mut T where T: ToSexp
}

deref_impl! {
    <T> ToSexp for Box<T> where T: ToSexp
}
