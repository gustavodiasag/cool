use std::collections::VecDeque;

use crate::{
    ast::{
        self, AstNode,
        bindings::{Cursor, Node, Tree},
    },
    language::Cool,
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
        let mut state = ConversionState::new();

        state.push_entry(root);
        // The idea is to build the internal AST from bottom-to-up and from
        // left-to-right.
        //
        // Once the array of children is built, the node itself can be
        // constructed until the root.
        loop {
            let node = state.last_node()?;
            self.cursor.reset(node);

            if self.cursor.goto_first_child() {
                let node = self.cursor.node();
                state.push_entry(node);
            } else {
                loop {
                    let (node, children) = state.pop_entry()?;

                    if let Ok(ast_node) = self.try_cast(&node, children) {
                        if state.is_empty() {
                            return match ast_node {
                                AstNode::Program(p) => Some(p),
                                _ => None,
                            };
                        }
                        state.last_children_mut()?.push_back(ast_node);
                    }
                    if let Some(next) = node.next_sibling() {
                        state.push_entry(next);
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

    fn convert_binop(&mut self, node: &Node) -> Option<ast::BinaryOp> {
        let rule = node.child(1)?.rule();

        match rule {
            Cool::Plus => Some(ast::BinaryOp::Add),
            Cool::Dash => Some(ast::BinaryOp::Sub),
            Cool::Star => Some(ast::BinaryOp::Mul),
            Cool::Slash => Some(ast::BinaryOp::Div),
            Cool::Lt => Some(ast::BinaryOp::Lt),
            Cool::Lte => Some(ast::BinaryOp::Lte),
            Cool::Eq => Some(ast::BinaryOp::Eq),
            _ => {
                self.error(Error::InvalidSyntax);
                None
            }
        }
    }
}

impl Converter<'_, '_> {
    fn try_cast(&mut self, node: &Node, mut children: VecDeque<AstNode>) -> Result<AstNode> {
        match node.rule() {
            Cool::SourceFile => {
                let program = ast::Program {
                    classes: self.consume_non_zero(children)?,
                };

                Ok(program.into())
            }
            Cool::ClassItem => {
                let class = ast::Class {
                    name: self.consume(&mut children)?,
                    inherits: self.consume_opt(&mut children),
                    features: self.consume(&mut children)?,
                };

                Ok(class.into())
            }
            Cool::FieldDeclarationList => {
                let features = ast::Features(self.consume_all(children)?);

                Ok(features.into())
            }
            Cool::AttributeDeclaration => {
                let attribute = ast::Attribute {
                    name: self.consume(&mut children)?,
                    ty: self.consume(&mut children)?,
                    initializer: self.consume_opt(&mut children),
                };

                Ok(ast::Feature::Attribute(attribute).into())
            }
            Cool::MethodDeclaration => {
                let method = ast::Method {
                    name: self.consume(&mut children)?,
                    params: self.consume(&mut children)?,
                    return_ty: self.consume(&mut children)?,
                    body: self.consume(&mut children)?,
                };

                Ok(ast::Feature::Method(method).into())
            }
            Cool::Parameters => {
                let params = ast::Params(self.consume_all(children)?);

                Ok(params.into())
            }
            Cool::Parameter => {
                let param = ast::Param {
                    name: self.consume(&mut children)?,
                    ty: self.consume(&mut children)?,
                };

                Ok(param.into())
            }
            Cool::AssignmentExpression => {
                let expr = ast::Expr::Assignment {
                    name: self.consume(&mut children)?,
                    right: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::DispatchExpression => {
                let expr = ast::Expr::Dispatch {
                    qualifier: self.consume_opt(&mut children),
                    method: self.consume(&mut children)?,
                    args: self.consume_all(children)?,
                };

                Ok(expr.into())
            }
            Cool::IfExpression => {
                let expr = ast::Expr::Conditional {
                    condition: Box::new(self.consume(&mut children)?),
                    consequence: Box::new(self.consume(&mut children)?),
                    alternative: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::WhileExpression => {
                let expr = ast::Expr::Repeat {
                    condition: Box::new(self.consume(&mut children)?),
                    body: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::Block => {
                let expr = ast::Expr::Block {
                    body: self.consume_non_zero(children)?,
                };

                Ok(expr.into())
            }
            Cool::LetExpression => {
                let expr = ast::Expr::Let {
                    bindings: self.consume(&mut children)?,
                    body: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::Bindings => {
                let bindings = ast::LetBindings(self.consume_non_zero(children)?);

                Ok(bindings.into())
            }
            Cool::Binding => {
                let binding = ast::LetBinding {
                    name: self.consume(&mut children)?,
                    ty: self.consume(&mut children)?,
                    right: self.consume_opt(&mut children),
                };

                Ok(binding.into())
            }
            Cool::CaseExpression => {
                let expr = ast::Expr::Case {
                    value: Box::new(self.consume(&mut children)?),
                    body: self.consume_all(children)?,
                };

                Ok(expr.into())
            }
            Cool::CaseArm => {
                let arm = ast::CaseArm {
                    pat: self.consume(&mut children)?,
                    value: Box::new(self.consume(&mut children)?),
                };

                Ok(arm.into())
            }
            Cool::CasePattern => {
                let pat = ast::CasePattern {
                    name: self.consume(&mut children)?,
                    ty: self.consume(&mut children)?,
                };

                Ok(pat.into())
            }
            Cool::NewExpression => {
                let expr = ast::Expr::New {
                    ty: self.consume(&mut children)?,
                };

                Ok(expr.into())
            }
            Cool::IsvoidExpression => {
                let expr = ast::Expr::Unary {
                    op: ast::UnaryOp::IsVoid,
                    right: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::UnaryExpression => {
                let expr = ast::Expr::Unary {
                    op: ast::UnaryOp::Complement,
                    right: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::NotExpression => {
                let expr = ast::Expr::Unary {
                    op: ast::UnaryOp::Not,
                    right: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::BinaryExpression => {
                let op = self.convert_binop(node).ok_or(())?;

                let expr = ast::Expr::Binary {
                    op,
                    left: Box::new(self.consume(&mut children)?),
                    right: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::ParenthesizedExpression => {
                let expr = ast::Expr::Paren {
                    value: Box::new(self.consume(&mut children)?),
                };

                Ok(expr.into())
            }
            Cool::StringLiteral => {
                let expr = ast::Expr::String(extract::string(self.src, node));

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
                let to_ast = |ident| {
                    if node.parent().unwrap().rule().is_expr() {
                        ast::Expr::Ident(ident).into()
                    } else {
                        ident.into()
                    }
                };
                self.convert_ident(node)
                    .map(|ident| {
                        if let Some(parent) = node.parent() {
                            if parent.rule().is_expr() {
                                ast::Expr::Ident(ident).into()
                            } else {
                                ident.into()
                            }
                        }
                    })
                    .ok_or(())
            }
            Cool::Error => {
                self.error(Error::InvalidSyntax);
                Err(())
            }
            _ => Err(()),
        }
    }

    fn consume<T>(&mut self, children: &mut VecDeque<AstNode>) -> Result<T>
    where
        T: TryFrom<AstNode, Error = Error>,
    {
        let child = children.pop_front().ok_or(())?;

        T::try_from(child).map_err(|err| self.error(err))
    }

    fn consume_opt<T>(&mut self, children: &mut VecDeque<AstNode>) -> Option<T>
    where
        T: TryFrom<AstNode, Error = Error>,
    {
        let child = children.pop_front()?;

        match T::try_from(child) {
            Ok(value) => Some(value),
            Err(Error::Unexpected(other)) => {
                children.push_back(other);
                None
            }
            Err(_) => unreachable!(),
        }
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

struct ConversionState<'a> {
    node_stack: Vec<Node<'a>>,
    child_stack: Vec<VecDeque<AstNode>>,
}

impl<'a> ConversionState<'a> {
    fn new() -> ConversionState<'a> {
        ConversionState {
            node_stack: Vec::new(),
            child_stack: Vec::new(),
        }
    }

    fn push_entry(&mut self, node: Node<'a>) {
        self.child_stack.push(VecDeque::new());
        self.node_stack.push(node);
    }

    fn pop_entry(&mut self) -> Option<(Node<'a>, VecDeque<AstNode>)> {
        let node = self.node_stack.pop();
        let children = self.child_stack.pop();

        node.zip(children)
    }

    fn last_node(&self) -> Option<&Node<'a>> {
        self.node_stack.last()
    }

    fn last_node_mut(&mut self) -> Option<&mut Node<'a>> {
        self.node_stack.last_mut()
    }

    fn last_children(&self) -> Option<&VecDeque<AstNode>> {
        self.child_stack.last()
    }

    fn last_children_mut(&mut self) -> Option<&mut VecDeque<AstNode>> {
        self.child_stack.last_mut()
    }

    fn is_empty(&self) -> bool {
        self.child_stack.is_empty()
    }
}

mod extract {
    use crate::ast::bindings::Node;

    pub fn trivial<T>(src: &[u8], node: &Node) -> T
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Debug,
    {
        lexeme(src, node).parse::<T>().unwrap()
    }

    pub fn string(src: &[u8], node: &Node) -> Box<str> {
        let content = node.child(1);

        match content {
            Some(c) => lexeme(src, &c).to_string().into_boxed_str(),
            _ => Box::from(""),
        }
    }

    pub fn lexeme<'a>(src: &'a [u8], node: &'a Node) -> &'a str {
        node.utf8_text(src).unwrap()
    }
}

#[derive(Debug)]
pub enum Error {
    Unexpected(AstNode),
    EmptyConstruct,
    InvalidSyntax,
}
