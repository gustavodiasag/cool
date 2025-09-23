use tree_sitter::Node as OtherNode;
use tree_sitter::Tree as OtherTree;
use tree_sitter::{Parser, TreeCursor};

#[derive(Clone, Debug)]
pub struct Tree(OtherTree);

impl Tree {
    pub fn new(code: &[u8]) -> Self {
        let mut parser = Parser::new();
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
pub struct Cursor<'a>(TreeCursor<'a>);

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
pub struct Node<'a>(OtherNode<'a>);

impl<'a> Node<'a> {
    pub fn has_error(&self) -> bool {
        self.0.has_error()
    }

    pub(crate) fn id(&self) -> usize {
        self.0.id()
    }

    pub(crate) fn kind(&self) -> &'static str {
        self.0.kind()
    }

    pub(crate) fn kind_id(&self) -> u16 {
        self.0.kind_id()
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    LtEq = 36,
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
    FieldIdentifier = 90,
    SourceFileRepeat1 = 91,
    FieldDeclarationListRepeat1 = 92,
    ParametersRepeat1 = 93,
    ArgumentsRepeat1 = 94,
    BlockRepeat1 = 95,
    LetExpressionRepeat1 = 96,
    CaseExpressionRepeat1 = 97,
    StringLiteralRepeat1 = 98,
    BlockCommentRepeat1 = 99,
    AliasFieldIdentifier = 100,
}

impl From<Cool> for &'static str {
    fn from(token: Cool) -> Self {
        match token {
            Cool::End => "end",
            Cool::Class => "class",
            Cool::Inherits => "inherits",
            Cool::Semi => ";",
            Cool::LBrace => "{",
            Cool::RBrace => "}",
            Cool::Colon => ":",
            Cool::LtDash => "<-",
            Cool::LParen => "(",
            Cool::Comma => ",",
            Cool::RParen => ")",
            Cool::Bool | Cool::Int | Cool::Io | Cool::Object | Cool::String | Cool::SelfType => {
                "primitive_type"
            }
            Cool::At => "@",
            Cool::Dot => ".",
            Cool::If => "if",
            Cool::Then => "then",
            Cool::Else => "else",
            Cool::Fi => "fi",
            Cool::While => "while",
            Cool::Loop => "loop",
            Cool::Pool => "pool",
            Cool::Let => "let",
            Cool::In => "in",
            Cool::Case => "case",
            Cool::Of => "of",
            Cool::Esac => "esac",
            Cool::EqGt => "=>",
            Cool::New => "new",
            Cool::Isvoid => "isvoid",
            Cool::Not => "not",
            Cool::Tilde => "~",
            Cool::LtEq => "<=",
            Cool::Lt => "<",
            Cool::Eq => "=",
            Cool::Plus => "+",
            Cool::Dash => "-",
            Cool::Star => "*",
            Cool::Slash => "/",
            Cool::True => "true",
            Cool::False => "false",
            Cool::IntegerLiteral => "integer_literal",
            Cool::DQuote | Cool::DQuote2 => "\"",
            Cool::EscapeSequence => "escape_sequence",
            Cool::DashDash => "--",
            Cool::InlineCommentToken1 => "inline_comment_token1",
            Cool::LParenStar => "(*",
            Cool::BlockCommentToken1 => "block_comment_token1",
            Cool::BlockCommentToken2 => "block_comment_token2",
            Cool::StarRparen => "*)",
            Cool::Identifier => "identifier",
            Cool::TypeIdentifier => "type_identifier",
            Cool::SelfIdentifier => "self",
            Cool::StringContent => "string_content",
            Cool::Error => "_error_sentinel",
            Cool::SourceFile => "source_file",
            Cool::ClassItem => "class_item",
            Cool::FieldDeclarationList => "field_declaration_list",
            Cool::AttributeDeclaration => "attribute_declaration",
            Cool::MethodDeclaration => "method_declaration",
            Cool::Parameters => "parameters",
            Cool::Parameter => "parameter",
            Cool::Type => "_type",
            Cool::Expression => "_expression",
            Cool::AssignmentExpression => "assignment_expression",
            Cool::DispatchExpression => "dispatch_expression",
            Cool::Arguments => "arguments",
            Cool::IfExpression => "if_expression",
            Cool::WhileExpression => "while_expression",
            Cool::Block => "block",
            Cool::LetExpression => "let_expression",
            Cool::CaseExpression => "case_expression",
            Cool::CaseArm => "case_arm",
            Cool::CasePattern => "case_pattern",
            Cool::NewExpression => "new_expression",
            Cool::IsvoidExpression => "isvoid_expression",
            Cool::NotExpression => "not_expression",
            Cool::UnaryExpression => "unary_expression",
            Cool::BinaryExpression => "binary_expression",
            Cool::ParenthesizedExpression => "parenthesized_expression",
            Cool::Literal => "_literal",
            Cool::BooleanLiteral => "boolean_literal",
            Cool::StringLiteral => "string_literal",
            Cool::InlineComment => "inline_comment",
            Cool::BlockComment => "block_comment",
            Cool::FieldIdentifier => "field_identifier",
            Cool::SourceFileRepeat1 => "source_file_repeat1",
            Cool::FieldDeclarationListRepeat1 => "field_declaration_list_repeat1",
            Cool::ParametersRepeat1 => "parameters_repeat1",
            Cool::ArgumentsRepeat1 => "arguments_repeat1",
            Cool::BlockRepeat1 => "block_repeat1",
            Cool::LetExpressionRepeat1 => "let_expression_repeat1",
            Cool::CaseExpressionRepeat1 => "case_expression_repeat1",
            Cool::StringLiteralRepeat1 => "string_literal_repeat1",
            Cool::BlockCommentRepeat1 => "block_comment_repeat1",
            Cool::AliasFieldIdentifier => "field_identifier",
        }
    }
}

impl from<u16> for cool {
    fn from(value: u16) -> Self {
        num::FromPrimitive::from_u16(x).unwrap_or(Self::Error)
    }
}
