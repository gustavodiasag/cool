use std::collections::VecDeque;

use crate::{
    ast::{
        AstNode, Class, Program, Type,
        bindings::{Cool Cursor, Node, NodeKind},
        error::Error,
    },
    util::interner::Interner,
};

pub type Result<T, E = ()> = std::result::Result<T, E>;

pub fn convert(_src: &str) -> Result<Program> {
    todo!()
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

    fn try_cast(&mut self, node: &Node, children: VecDeque<AstNode>) -> Result<AstNode> {
        use Cool::*;

        let ast_node = match node.kind_id().into() {
            SourceFile => {
                let classes = children
                    .into_iter()
                    .filter_map(|child| {
                        if let AstNode::Class(class) = node {
                            Some(class)
                        }
                        None
                    })
                    .collect();

                AstNode::Program(classes)
            }
            ClassItem => {
                let AstNode::Type(name) = children.pop_front().unwrap() else {
                    panic!();
                };
                let AstNode::Type(inherits) = children.pop_front() else {
                    None
                };
            }
            _ => todo!(),
        };

        Ok(ast_node)
    }
}
