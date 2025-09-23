use crate::util::interner::InternedIdx;

pub mod bindings;
pub mod converter;
mod error;

#[derive(Debug)]
pub enum AstNode {
    Program(Program),
    Class(Class),
    Features(Features),
    Feature(Feature),
    Attribute(Attribute),
    Type(Type),
    Ident(Ident),
}

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
    pub initializer: Option<Expression>,
}

#[derive(Debug)]
pub struct Method {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_ty: Type,
    pub body: Expression,
}

#[derive(Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug)]
pub enum Expression {
    Assignment {
        name: Ident,
        right: Box<Expression>,
    },
    Dispatch {
        qualifier: Option<Qualifier>,
        method: Ident,
        args: Vec<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Box<Expression>,
    },
    Repeat {
        condition: Box<Expression>,
        body: Box<Expression>,
    },
    Block {
        body: Vec<Expression>,
    },
    Let {
        bindings: Vec<Binding>,
        body: Box<Expression>,
    },
    Case {
        value: Box<Expression>,
        arms: Vec<CaseArm>,
    },
    New {
        ty: Type,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Paren {
        value: Box<Expression>,
    },
    Ident(Ident),
    Int(u64),
    Bool(bool),
}

#[derive(Debug)]
pub struct Qualifier {
    value: Box<Expression>,
    parent: Option<Type>,
}

#[derive(Debug)]
pub struct Binding {
    name: Ident,
    ty: Type,
    right: Option<Expression>,
}

#[derive(Debug)]
pub struct CaseArm {
    name: Ident,
    ty: Type,
    value: Box<Expression>,
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
