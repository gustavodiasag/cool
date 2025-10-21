use crate::{ast::macros::ast_node, util::interner::InternedIdx};

pub mod bindings;
pub mod converter;

mod macros;

ast_node! {
    #[derive(Debug)]
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
        LetBindings(LetBindings),
        LetBinding(LetBinding),
        CaseArm(CaseArm),
        CasePattern(CasePattern),
        UnaryOp(UnaryOp),
        BinaryOp(BinaryOp),
        Type(Type),
        Ident(Ident),
    }
}

#[derive(Debug, Default)]
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
    pub params: Params,
    pub return_ty: Type,
    pub body: Expr,
}

#[derive(Debug)]
pub struct Params(pub Vec<Param>);

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
        bindings: LetBindings,
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
    String(Box<str>),
    Int(u64),
    Bool(bool),
}

#[derive(Debug)]
pub struct Qualifier {
    value: Box<Expr>,
    parent: Option<Type>,
}

#[derive(Debug)]
pub struct LetBindings(pub Vec<LetBinding>);

#[derive(Debug)]
pub struct LetBinding {
    name: Ident,
    ty: Type,
    right: Option<Expr>,
}

#[derive(Debug)]
pub struct CaseArm {
    pat: CasePattern,
    value: Box<Expr>,
}

#[derive(Debug)]
pub struct CasePattern {
    name: Ident,
    ty: Type,
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
