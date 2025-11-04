use sexp_derive::Sexp;

use crate::test_utils::assert_sexp;

mod test_utils;

#[derive(Sexp)]
#[sexp(rename = "unit")]
struct Unit;

#[test]
fn test_unit_struct() {
    assert_sexp(&Unit, "unit");
}

#[test]
fn test_unit_variant() {
    #[derive(Sexp)]
    #[sexp(rename = "test")]
    struct Test {
        my_enum: Vec<MyEnum>,
        my_bool: bool,
    }

    #[derive(Sexp)]
    enum MyEnum {
        Struct { foo: i32, bar: f32 },
    }

    assert_sexp(
        &Test {
            my_enum: &[
                MyEnum::Struct { foo: 4, bar: 5.3 },
                MyEnum::Struct { foo: 80, bar: 10.0 },
            ],
            my_bool: false,
        },
        r#"
(test
  (my_enum: 
    (struct
      (foo: 4)
      (bar: 5.3))
    (struct
      (foo: 80)
      (bar: 10.0)))
  (my_bool: false))
"#,
    );
}
