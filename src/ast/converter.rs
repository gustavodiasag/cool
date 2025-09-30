use std::collections::VecDeque;

use crate::{
    ast::{
        self, AstNode,
        bindings::{Cool, Cursor, Node, Tree},
    },
    util::{interner::Interner, span::Spanned},
};

type Result<T, E = ()> = std::result::Result<T, E>;

pub type ConversionResult<T> = Result<T, (T, Vec<Spanned<Error>>)>;

pub fn convert(src: &[u8], tree: &Tree, interner: &mut Interner) -> ConversionResult<ast::Program> {
    let mut converter = Converter::new(src, tree, interner);
    let program = converter.convert().unwrap_or_default();

    if converter.errors.is_empty() {
        Ok(program)
    } else {
        Err((program, converter.errors))
    }
}

struct Converter<'src, 'i> {
    src: &'src [u8],
    interner: &'i mut Interner,
    cursor: Cursor<'src>,
    errors: Vec<Spanned<Error>>,
}

impl Converter<'_, '_> {
    fn new<'src, 'i>(
        src: &'src [u8],
        tree: &'src Tree,
        interner: &'i mut Interner,
    ) -> Converter<'src, 'i> {
        let cursor = tree.get_root().cursor();
        Converter {
            src,
            interner,
            cursor,
            errors: Vec::default(),
        }
    }

    fn convert(&mut self) -> Option<ast::Program> {
        let root = self.cursor.node();

        let mut nodes: Vec<Node> = Vec::with_capacity(root.descendant_count());
        let mut children: Vec<VecDeque<AstNode>> = Vec::new();

        nodes.push(root);
        children.push(VecDeque::new());
        // The idea is to build the internal AST from bottom-to-up and from
        // left-to-right.
        //
        // Once the array of children is built, the node itself can be
        // constructed until the root.
        loop {
            let node = nodes.last()?;
            self.cursor.reset(node);

            if self.cursor.goto_first_child() {
                let node = self.cursor.node();
                children.push(VecDeque::with_capacity(node.child_count()));
                nodes.push(node);
            } else {
                loop {
                    let node = nodes.pop()?;
                    let node_children = children.pop()?;

                    if let Ok(ast_node) = self.try_cast(&node, node_children) {
                        if children.is_empty() {
                            let AstNode::Program(program) = ast_node else {
                                return None;
                            };
                            return Some(program);
                        }
                        children.last_mut()?.push_back(ast_node);
                    }
                    if let Some(next) = node.next_sibling() {
                        children.push(VecDeque::with_capacity(next.child_count()));
                        nodes.push(next);

                        break;
                    }
                }
            }
        }
    }

    fn convert_type(&mut self, node: &Node) -> Option<ast::Type> {
        self.convert_ident(node).map(ast::Type)
    }

    fn convert_ident(&mut self, node: &Node) -> Option<ast::Ident> {
        let name = extract::lexeme(self.src, node);
        // If the tree-sitter parser is able to recover from erroneous text by
        // inserting a missing token and then reducing, it will do so in the
        // final tree, so long as it has the lowest error cost.
        //
        // The problem here is that a missing node doesn't have its own
        // representation, such as `Cool::Error`, meaning that this redundant
        // check must occur.
        if name.is_empty() {
            self.error(Error::InvalidSyntax);
            return None;
        }
        Some(ast::Ident {
            name: self.interner.intern(name),
        })
    }

    fn try_cast(&mut self, node: &Node, mut children: VecDeque<AstNode>) -> Result<AstNode> {
        match node.kind() {
            Cool::SourceFile => {
                let program = ast::Program {
                    classes: self.consume_non_zero(children)?,
                };

                Ok(program.into())
            }
            Cool::ClassItem => {
                let class = ast::Class {
                    name: self.consume_as(&mut children)?,
                    inherits: self.consume_opt(&mut children),
                    features: self.consume_as(&mut children)?,
                };

                Ok(class.into())
            }
            Cool::FieldDeclarationList => {
                let features = ast::Features(self.consume_all(children)?);

                Ok(features.into())
            }
            Cool::AttributeDeclaration => {
                let attribute = ast::Attribute {
                    name: self.consume_as(&mut children)?,
                    ty: self.consume_as(&mut children)?,
                    initializer: self.consume_opt(&mut children),
                };

                Ok(ast::Feature::Attribute(attribute).into())
            }
            Cool::MethodDeclaration => {
                let method = ast::Method {
                    name: self.consume_as(&mut children)?,
                    params: self.consume_as(&mut children)?,
                    return_ty: self.consume_as(&mut children)?,
                    body: self.consume_as(&mut children)?,
                };

                Ok(ast::Feature::Method(method).into())
            }
            Cool::Parameters => {
                let params = ast::Params(self.consume_all(children)?);

                Ok(params.into())
            }
            Cool::Parameter => {
                let param = ast::Param {
                    name: self.consume_as(&mut children)?,
                    ty: self.consume_as(&mut children)?,
                };

                Ok(param.into())
            }
            Cool::AssignmentExpression => {
                let expr = ast::Expr::Assignment {
                    name: self.consume_as(&mut children)?,
                    right: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::DispatchExpression => {
                let expr = ast::Expr::Dispatch {
                    qualifier: self.consume_opt(&mut children),
                    method: self.consume_as(&mut children)?,
                    args: self.consume_all(children)?,
                };

                Ok(expr.into())
            }
            Cool::IfExpression => {
                let expr = ast::Expr::Conditional {
                    condition: Box::new(self.consume_as(&mut children)?),
                    consequence: Box::new(self.consume_as(&mut children)?),
                    alternative: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::WhileExpression => {
                let expr = ast::Expr::Repeat {
                    condition: Box::new(self.consume_as(&mut children)?),
                    body: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::Block => {
                let expr = ast::Expr::Block {
                    body: self.consume_non_zero(children)?,
                };

                Ok(expr.into())
            }
            Cool::CaseExpression => {
                let expr = ast::Expr::Case {
                    value: Box::new(self.consume_as(&mut children)?),
                    body: self.consume_all(children)?,
                };

                Ok(expr.into())
            }
            Cool::CaseArm => {
                let arm = ast::CaseArm {
                    pat: self.consume_as(&mut children)?,
                    value: Box::new(self.consume_as(&mut children)?),
                };

                Ok(arm.into())
            }
            Cool::CasePattern => {
                let pat = ast::CasePattern {
                    name: self.consume_as(&mut children)?,
                    ty: self.consume_as(&mut children)?,
                };

                Ok(pat.into())
            }
            Cool::NewExpression => {
                let expr = ast::Expr::New {
                    ty: self.consume_as(&mut children)?,
                };

                Ok(expr.into())
            }
            Cool::IsvoidExpression => {
                let expr = ast::Expr::Unary {
                    op: ast::UnaryOp::IsVoid,
                    right: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::UnaryExpression => {
                let expr = ast::Expr::Unary {
                    op: ast::UnaryOp::Complement,
                    right: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::NotExpression => {
                let expr = ast::Expr::Unary {
                    op: ast::UnaryOp::Not,
                    right: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::BinaryExpression => {
                let expr = ast::Expr::Binary {
                    op: node.child(1).unwrap().kind().into(),
                    left: Box::new(self.consume_as(&mut children)?),
                    right: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::ParenthesizedExpression => {
                let expr = ast::Expr::Paren {
                    value: Box::new(self.consume_as(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::IntegerLiteral => {
                let expr = ast::Expr::Int(extract::trivial(self.src, node));

                Ok(expr.into())
            }
            Cool::BooleanLiteral => {
                let expr = ast::Expr::Bool(extract::trivial(self.src, node));

                Ok(expr.into())
            }
            Cool::TypeIdentifier
            | Cool::Bool
            | Cool::Int
            | Cool::Io
            | Cool::Object
            | Cool::String
            | Cool::SelfType => {
                let ty = self.convert_type(node).ok_or(())?;

                Ok(ty.into())
            }
            Cool::Identifier | Cool::FieldIdentifier | Cool::SelfIdentifier => {
                let ident = self.convert_ident(node).ok_or(())?;

                Ok(ident.into())
            }
            Cool::Error => {
                self.error(Error::InvalidSyntax);
                Err(())
            }
            _ => Err(()),
        }
    }
}

impl Converter<'_, '_> {
    fn consume_opt<T>(&mut self, children: &mut VecDeque<AstNode>) -> Option<T>
    where
        T: TryFrom<AstNode, Error = Error>,
    {
        let child = children.pop_front()?;

        match T::try_from(child) {
            Ok(val) => Some(val),
            Err(Error::Unexpected(other)) => {
                children.push_back(other);
                None
            }
            Err(_) => unreachable!(),
        }
    }

    fn consume_as<T>(&mut self, children: &mut VecDeque<AstNode>) -> Result<T>
    where
        T: TryFrom<AstNode, Error = Error>,
    {
        let child = children.pop_front().ok_or(())?;

        T::try_from(child).map_err(|err| self.error(err))
    }

    fn consume_all<T>(&mut self, children: VecDeque<AstNode>) -> Result<Vec<T>>
    where
        T: TryFrom<AstNode, Error = Error>,
    {
        children
            .into_iter()
            .map(|child| T::try_from(child).map_err(|err| self.error(err)))
            .collect()
    }

    fn consume_non_zero<T>(&mut self, children: VecDeque<AstNode>) -> Result<Vec<T>>
    where
        T: TryFrom<AstNode, Error = Error>,
    {
        self.consume_all(children).and_then(|res| {
            if res.is_empty() {
                self.error(Error::EmptyConstruct);
                Err(())
            } else {
                Ok(res)
            }
        })
    }

    fn error(&mut self, error: Error) {
        self.errors.push(Spanned {
            inner: error,
            span: self.cursor.node().span(),
        });
    }
}

mod extract {
    use crate::ast::bindings::{Cool, Node};

    pub fn trivial<T>(src: &[u8], node: &Node) -> T
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Debug,
    {
        lexeme(src, node).parse::<T>().unwrap()
    }

    pub fn lexeme<'a>(src: &'a [u8], node: &'a Node) -> &'a str {
        node.utf8_text(src).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unexpected(AstNode),
    EmptyConstruct,
    InvalidSyntax,
}
