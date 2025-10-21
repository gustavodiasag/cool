use num_derive::FromPrimitive;

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
    Bindings = 76,
    Binding = 77,
    CaseExpression = 78,
    CaseArm = 79,
    CasePattern = 80,
    NewExpression = 81,
    IsvoidExpression = 82,
    NotExpression = 83,
    UnaryExpression = 84,
    BinaryExpression = 85,
    ParenthesizedExpression = 86,
    Literal = 87,
    BooleanLiteral = 88,
    StringLiteral = 89,
    InlineComment = 90,
    BlockComment = 91,
    SourceFileRepeat1 = 92,
    FieldDeclarationListRepeat1 = 93,
    ParametersRepeat1 = 94,
    ArgumentsRepeat1 = 95,
    BlockRepeat1 = 96,
    LetExpressionRepeat1 = 97,
    CaseExpressionRepeat1 = 98,
    StringLiteralRepeat1 = 99,
    BlockCommentRepeat1 = 100,
    FieldIdentifier = 101,
}

impl Cool {
    pub fn is_expr(&self) -> bool {
        matches!(
            self,
            Cool::IntegerLiteral
                | Cool::StringContent
                | Cool::Expression
                | Cool::AssignmentExpression
                | Cool::DispatchExpression
                | Cool::IfExpression
                | Cool::WhileExpression
                | Cool::Block
                | Cool::LetExpression
                | Cool::CaseExpression
                | Cool::CasePattern
                | Cool::NewExpression
                | Cool::UnaryExpression
                | Cool::BinaryExpression
                | Cool::ParenthesizedExpression
                | Cool::Literal
                | Cool::BooleanLiteral
                | Cool::StringLiteral
        )
    }
}

impl From<u16> for Cool {
    #[inline(always)]
    fn from(x: u16) -> Self {
        num_traits::FromPrimitive::from_u16(x).unwrap_or(Self::Error)
    }
}
