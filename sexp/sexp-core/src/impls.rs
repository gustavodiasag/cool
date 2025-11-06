use std::collections::VecDeque;

use crate::{Result, SexpSerializer, ToSexp};

macro_rules! primitive_impls {
    (
        $($func:ident => $($ty:ty),+);+ $(;)?
    ) => {
        $(
            $(
                impl ToSexp for $ty {
                    fn to_sexp<S>(&self, s: &mut S) -> Result<()>
                    where
                        S: SexpSerializer,
                    {
                        s.$func(*self)
                    }
                }
            )+
        )+
    };
}

primitive_impls! {
    serialize_bool => bool;
    serialize_num => i8, i16, i32, i64, i128, u8, u16, u32, u64, u128;
    serialize_float => f32, f64;
    serialize_char => char;
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
