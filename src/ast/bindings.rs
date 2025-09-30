use num_derive::FromPrimitive;

use crate::util::span::Span;

#[derive(Clone, Debug)]
pub struct Tree(tree_sitter::Tree);

impl Tree {
    pub fn new(code: &[u8]) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_cool::LANGUAGE.into())
            .unwrap();

        Self(parser.parse(code, None).unwrap())
    }

    pub fn get_root(&self) -> Node {
        Node(self.0.root_node())
    }
}

#[derive(Clone)]
pub struct Cursor<'a>(tree_sitter::TreeCursor<'a>);

impl<'a> Cursor<'a> {
    pub(crate) fn reset(&mut self, node: &Node<'a>) {
        self.0.reset(node.0);
    }

    pub(crate) fn goto_next_sibling(&mut self) -> bool {
        self.0.goto_next_sibling()
    }

    pub(crate) fn goto_first_child(&mut self) -> bool {
        self.0.goto_first_child()
    }

    pub(crate) fn node(&self) -> Node<'a> {
        Node(self.0.node())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Node<'a>(tree_sitter::Node<'a>);

impl<'a> Node<'a> {
    pub(crate) fn has_error(&self) -> bool {
        self.0.has_error()
    }

    pub fn is_missing(&self) -> bool {
        self.0.is_missing()
    }

    pub(crate) fn id(&self) -> usize {
        self.0.id()
    }

    pub(crate) fn rule(&self) -> &'static str {
        self.0.kind()
    }

    pub(crate) fn kind(&self) -> Cool {
        self.0.kind_id().into()
    }

    pub(crate) fn utf8_text(&self, data: &'a [u8]) -> Option<&'a str> {
        self.0.utf8_text(data).ok()
    }

    pub(crate) fn start_byte(&self) -> usize {
        self.0.start_byte()
    }

    pub(crate) fn end_byte(&self) -> usize {
        self.0.end_byte()
    }

    pub(crate) fn span(&self) -> Span {
        let (srow, scol) = self.start_position();
        let (erow, ecol) = self.end_position();

        (srow + 1, scol + 1, erow + 1, ecol + 1)
    }

    pub(crate) fn start_position(&self) -> (usize, usize) {
        let temp = self.0.start_position();
        (temp.row, temp.column)
    }

    pub(crate) fn end_position(&self) -> (usize, usize) {
        let temp = self.0.end_position();
        (temp.row, temp.column)
    }

    pub(crate) fn start_row(&self) -> usize {
        self.0.start_position().row
    }

    pub(crate) fn end_row(&self) -> usize {
        self.0.end_position().row
    }

    pub(crate) fn parent(&self) -> Option<Node<'a>> {
        self.0.parent().map(Node)
    }

    #[inline(always)]
    pub(crate) fn has_sibling(&self, id: u16) -> bool {
        self.0.parent().is_some_and(|parent| {
            self.0
                .children(&mut parent.walk())
                .any(|child| child.kind_id() == id)
        })
    }

    pub(crate) fn previous_sibling(&self) -> Option<Node<'a>> {
        self.0.prev_sibling().map(Node)
    }

    pub(crate) fn next_sibling(&self) -> Option<Node<'a>> {
        self.0.next_sibling().map(Node)
    }

    #[inline(always)]
    pub(crate) fn is_child(&self, id: u16) -> bool {
        self.0
            .children(&mut self.0.walk())
            .any(|child| child.kind_id() == id)
    }

    pub fn descendant_count(&self) -> usize {
        self.0.descendant_count()
    }

    pub(crate) fn child_count(&self) -> usize {
        self.0.child_count()
    }

    pub(crate) fn child_by_field_name(&self, name: &str) -> Option<Node> {
        self.0.child_by_field_name(name).map(Node)
    }

    pub(crate) fn child(&self, pos: usize) -> Option<Node<'a>> {
        self.0.child(pos).map(Node)
    }

    pub(crate) fn children(&self) -> impl ExactSizeIterator<Item = Node<'a>> + use<'a> {
        let mut cursor = self.cursor();
        cursor.goto_first_child();
        (0..self.child_count()).map(move |_| {
            let result = cursor.node();
            cursor.goto_next_sibling();
            result
        })
    }

    pub(crate) fn cursor(&self) -> Cursor<'a> {
        Cursor(self.0.walk())
    }

    #[allow(dead_code)]
    pub(crate) fn get_parent(&self, level: usize) -> Option<Node<'a>> {
        let mut level = level;
        let mut node = *self;
        while level != 0 {
            if let Some(parent) = node.parent() {
                node = parent;
            } else {
                return None;
            }
            level -= 1;
        }

        Some(node)
    }

    pub(crate) fn has_ancestors(&self, typ: fn(&Node) -> bool, typs: fn(&Node) -> bool) -> bool {
        let mut res = false;
        let mut node = *self;
        if let Some(parent) = node.parent() {
            if typ(&parent) {
                node = parent;
            }
        }
        if let Some(parent) = node.parent() {
            if typs(&parent) {
                res = true;
            }
        }
        res
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum Cool {
    End = 0,
    Class = 1,
    Inherits = 2,
    Semi = 3,
    LBrace = 4,
    RBrace = 5,
    Colon = 6,
    LtDash = 7,
    LParen = 8,
    Comma = 9,
    RParen = 10,
    Bool = 11,
    Int = 12,
    Io = 13,
    Object = 14,
    String = 15,
    SelfType = 16,
    At = 17,
    Dot = 18,
    If = 19,
    Then = 20,
    Else = 21,
    Fi = 22,
    While = 23,
    Loop = 24,
    Pool = 25,
    Let = 26,
    In = 27,
    Case = 28,
    Of = 29,
    Esac = 30,
    EqGt = 31,
    New = 32,
    Isvoid = 33,
    Not = 34,
    Tilde = 35,
    Lte = 36,
    Lt = 37,
    Eq = 38,
    Plus = 39,
    Dash = 40,
    Star = 41,
    Slash = 42,
    True = 43,
    False = 44,
    IntegerLiteral = 45,
    DQuote = 46,
    DQuote2 = 47,
    EscapeSequence = 48,
    DashDash = 49,
    InlineCommentToken1 = 50,
    LParenStar = 51,
    BlockCommentToken1 = 52,
    BlockCommentToken2 = 53,
    StarRparen = 54,
    Identifier = 55,
    TypeIdentifier = 56,
    SelfIdentifier = 57,
    StringContent = 58,
    Error = 59,
    SourceFile = 60,
    ClassItem = 61,
    FieldDeclarationList = 62,
    AttributeDeclaration = 63,
    MethodDeclaration = 64,
    Parameters = 65,
    Parameter = 66,
    Type = 67,
    Expression = 68,
    AssignmentExpression = 69,
    DispatchExpression = 70,
    Arguments = 71,
    IfExpression = 72,
    WhileExpression = 73,
    Block = 74,
    LetExpression = 75,
    CaseExpression = 76,
    CaseArm = 77,
    CasePattern = 78,
    NewExpression = 79,
    IsvoidExpression = 80,
    NotExpression = 81,
    UnaryExpression = 82,
    BinaryExpression = 83,
    ParenthesizedExpression = 84,
    Literal = 85,
    BooleanLiteral = 86,
    StringLiteral = 87,
    InlineComment = 88,
    BlockComment = 89,
    AliasFieldIdentifier = 90,
    SourceFileRepeat1 = 91,
    FieldDeclarationListRepeat1 = 92,
    ParametersRepeat1 = 93,
    ArgumentsRepeat1 = 94,
    BlockRepeat1 = 95,
    LetExpressionRepeat1 = 96,
    CaseExpressionRepeat1 = 97,
    StringLiteralRepeat1 = 98,
    BlockCommentRepeat1 = 99,
    FieldIdentifier = 100,
}

impl From<u16> for Cool {
    #[inline(always)]
    fn from(x: u16) -> Self {
        num_traits::FromPrimitive::from_u16(x).unwrap_or(Self::Error)
    }
}
