use crate::util::interner::InternedIdx;

pub mod bindings;
pub mod converter;
mod error;
mod macros;

#[derive(Debug)]
pub struct Program {
    pub classes: Vec<Class>,
}

#[derive(Debug)]
pub struct Class {
    pub name: Type,
    pub inherits: Option<Type>,
    pub features: Features,
}

#[derive(Debug)]
pub struct Features(pub Vec<Feature>);

#[derive(Debug)]
pub enum Feature {
    Attribute(Attribute),
    Method(Method),
}

#[derive(Debug)]
pub struct Attribute {
    pub name: Ident,
    pub ty: Type,
    pub initializer: Option<Expr>,
}

#[derive(Debug)]
pub struct Method {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_ty: Type,
    pub body: Expr,
}

#[derive(Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug)]
pub enum Expr {
    Assignment {
        name: Ident,
        right: Box<Expr>,
    },
    Dispatch {
        qualifier: Option<Qualifier>,
        method: Ident,
        args: Vec<Expr>,
    },
    Conditional {
        condition: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>,
    },
    Repeat {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    Block {
        body: Vec<Expr>,
    },
    Let {
        bindings: Vec<Binding>,
        body: Box<Expr>,
    },
    Case {
        value: Box<Expr>,
        arms: Vec<CaseArm>,
    },
    New {
        ty: Type,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Paren {
        value: Box<Expr>,
    },
    Ident(Ident),
    Int(u64),
    Bool(bool),
}

#[derive(Debug)]
pub struct Qualifier {
    value: Box<Expr>,
    parent: Option<Type>,
}

#[derive(Debug)]
pub struct Binding {
    name: Ident,
    ty: Type,
    right: Option<Expr>,
}

#[derive(Debug)]
pub struct CaseArm {
    name: Ident,
    ty: Type,
    value: Box<Expr>,
}

#[derive(Debug)]
pub enum UnaryOp {
    IsVoid,
    Complement,
    Not,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Eq,
}

#[derive(Debug)]
pub struct Type(pub Ident);

#[derive(Debug)]
pub struct Ident {
    pub name: InternedIdx,
}
