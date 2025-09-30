use crate::{
    ast::{bindings::Cool, macros::ast_node},
    util::interner::InternedIdx,
};

pub mod bindings;
pub mod converter;

mod macros;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub classes: Vec<Class>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Class {
    pub name: Type,
    pub inherits: Option<Type>,
    pub features: Features,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Features(pub Vec<Feature>);

#[derive(Debug, PartialEq, Eq)]
pub enum Feature {
    Attribute(Attribute),
    Method(Method),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: Ident,
    pub ty: Type,
    pub initializer: Option<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Method {
    pub name: Ident,
    pub params: Params,
    pub return_ty: Type,
    pub body: Expr,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Params(pub Vec<Param>);

#[derive(Debug, PartialEq, Eq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Eq)]
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
        body: Vec<CaseArm>,
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

#[derive(Debug, PartialEq, Eq)]
pub struct Qualifier {
    value: Box<Expr>,
    parent: Option<Type>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Binding {
    name: Ident,
    ty: Type,
    right: Option<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CaseArm {
    pat: CasePattern,
    value: Box<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CasePattern {
    name: Ident,
    ty: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOp {
    IsVoid,
    Complement,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Eq,
}

impl From<bindings::Cool> for BinaryOp {
    fn from(value: Cool) -> Self {
        match value {
            Cool::Plus => BinaryOp::Add,
            Cool::Dash => BinaryOp::Sub,
            Cool::Star => BinaryOp::Mul,
            Cool::Slash => BinaryOp::Div,
            Cool::Lt => BinaryOp::Lt,
            Cool::Lte => BinaryOp::Lte,
            Cool::Eq => BinaryOp::Eq,
            _ => {
                println!("{:?}", value);
                todo!()
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Type(pub Ident);

#[derive(Debug, PartialEq, Eq)]
pub struct Ident {
    pub name: InternedIdx,
}

ast_node! {
    #[derive(Debug, PartialEq, Eq)]
    pub enum AstNode {
        Program(Program),
        Class(Class),
        Features(Features),
        Feature(Feature),
        Attribute(Attribute),
        Method(Method),
        Params(Params),
        Param(Param),
        Expr(Expr),
        Qualifier(Qualifier),
        CaseArm(CaseArm),
        CasePattern(CasePattern),
        UnaryOp(UnaryOp),
        BinaryOp(BinaryOp),
        Type(Type),
        Ident(Ident),
    }
}
