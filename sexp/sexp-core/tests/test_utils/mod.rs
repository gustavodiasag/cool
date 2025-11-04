use sexp_core::{SexpSerializerImpl, ToSexp};

pub fn assert_sexp<T: ToSexp>(value: &T, expected: &str) {
    let mut buf = Vec::new();

    let mut serializer = SexpSerializerImpl::new(&mut buf);

    match value.to_sexp(&mut serializer) {
        Ok(()) => {}
        Err(err) => panic!("failed generating s-expression for value: {}", err),
    }
    let actual = String::from_utf8(buf).unwrap();

    println!("{actual}");

    assert_eq!(actual, expected)
}
