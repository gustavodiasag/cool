use crate::ast;

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
                type Error = ();

                fn try_from(value: $name) -> Result<$ty, Self::Error> {
                    match value {
                        $name::$variant(v) => Ok(v),
                        _ => Err(())
                    }
                }
            }
        )*
    };
}

ast_node! {
    #[derive(Debug)]
    pub enum AstNode {
        Program(ast::Program),
        Class(ast::Class),
        Features(ast::Features),
        Feature(ast::Feature),
        Attribute(ast::Attribute),
        Expr(ast::Expr),
        Type(ast::Type),
        Ident(ast::Ident),
    }
}
