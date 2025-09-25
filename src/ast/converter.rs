use std::collections::VecDeque;

use crate::{
    ast::{
        self, Program,
        bindings::{Cool, Cursor, Node, Tree},
        error::Error,
        macros::AstNode,
    },
    util::interner::Interner,
};

pub type Result<T, E = ()> = std::result::Result<T, E>;

pub fn convert(src: &[u8]) -> Result<Program> {
    let mut interner = Interner::with_capacity(1_024);
    let tree = Tree::new(src);
    let cursor = tree.get_root().cursor();

    Converter::new(src, &mut interner, cursor).convert()
}

struct Converter<'src, 'i, 'c> {
    src: &'src [u8],
    interner: &'i mut Interner,
    cursor: Cursor<'c>,
    errors: VecDeque<Error>,
}

impl Converter<'_, '_, '_> {
    fn new<'src, 'i, 'c>(
        src: &'src [u8],
        interner: &'i mut Interner,
        cursor: Cursor<'c>,
    ) -> Converter<'src, 'i, 'c> {
        Converter {
            src,
            interner,
            cursor,
            errors: VecDeque::default(),
        }
    }

    fn convert(&mut self) -> Result<Program> {
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
            let node = nodes.last().unwrap();
            self.cursor.reset(node);

            if self.cursor.goto_first_child() {
                let node = self.cursor.node();
                children.push(VecDeque::with_capacity(node.child_count()));
                nodes.push(node);
            } else {
                loop {
                    let node = nodes.pop().unwrap();
                    let node_children = children.pop().unwrap();

                    if let Ok(ast_node) = self.try_cast(&node, node_children) {
                        if children.is_empty() {
                            let AstNode::Program(program) = ast_node else {
                                return Err(());
                            };
                            return Ok(program);
                        }
                        children.last_mut().unwrap().push_back(ast_node);
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

    fn try_cast(&mut self, node: &Node, mut children: VecDeque<AstNode>) -> Result<AstNode> {
        match node.kind_id().into() {
            Cool::SourceFile => {
                let classes = children
                    .into_iter()
                    .filter_map(|child| child.try_into().ok())
                    .collect();

                Ok(ast::Program { classes }.into())
            }
            Cool::ClassItem => {
                let class = ast::Class {
                    name: consume(children.pop_front())?,
                    inherits: consume(children.pop_front()).ok(),
                    features: consume(children.pop_front())?,
                };

                Ok(class.into())
            }
            Cool::FieldDeclarationList => {
                let features = children
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_>>()?;

                Ok(ast::Features(features).into())
            }
            Cool::AttributeDeclaration => {
                let attribute = ast::Attribute {
                    name: consume(children.pop_front())?,
                    ty: consume(children.pop_front())?,
                    initializer: consume(children.pop_front()).ok(),
                };

                Ok(ast::Feature::Attribute(attribute).into())
            }
            Cool::AliasFieldIdentifier | Cool::FieldIdentifier => {
                let name = node.utf8_text(self.src).unwrap();

                Ok(ast::Ident {
                    name: self.interner.intern(name),
                }
                .into())
            }
            Cool::TypeIdentifier
            | Cool::Bool
            | Cool::Int
            | Cool::Io
            | Cool::Object
            | Cool::String
            | Cool::SelfType => {
                let name = node.utf8_text(self.src).unwrap();
                let ident = ast::Ident {
                    name: self.interner.intern(name),
                };

                Ok(ast::Type(ident).into())
            }
            _ => Err(()),
        }
    }
}

fn consume<T>(maybe: Option<AstNode>) -> Result<T>
where
    T: TryFrom<AstNode, Error = ()>,
{
    let node = maybe.ok_or(())?;
    T::try_from(node)
}
