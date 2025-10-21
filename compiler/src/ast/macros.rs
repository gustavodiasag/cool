macro_rules! ast_node {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $variant:ident($ty:ty)
            ),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant($ty)),*
        }

        $(
            impl From<$ty> for $name {
                fn from(node: $ty) -> Self {
                    $name::$variant(node)
                }
            }

            impl TryFrom<$name> for $ty {
                type Error = $crate::ast::converter::Error;

                fn try_from(value: $name) -> Result<$ty, Self::Error> {
                    match value {
                        $name::$variant(v) => Ok(v),
                        other => Err(
                            $crate::ast::converter::Error::Unexpected(other)
                        )
                    }
                }
            }
        )*
    };
}

pub(crate) use ast_node;
