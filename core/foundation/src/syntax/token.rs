#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// "# comment"
    LineComment,

    /// Any whitespace character sequence.
    Whitespace,

    // Literal
    /// Integer literal. e.g. "42"
    Int { base: NumBase, empty_int: bool },
    /// Float literal. e.g. "3.14"
    Float { base: NumBase, empty_exponent: bool },
    /// String literal. e.g. "\"foo\""
    String {
        terminated: bool,
        quote_kind: QuoteKind,
    },
    /// "true"
    True,
    /// "false"
    False,
    /// "nil"
    Nil,

    /// Keyword (reserved identifiers)
    /// "var"
    Var,
    /// "func"
    Func,
    /// "if"
    If,
    /// "then"
    Then,
    /// "elif"
    Elif,
    /// "else"
    Else,
    /// "for"
    For,
    /// "while"
    While,
    /// "in"
    In,
    /// "do"
    Do,
    /// "end"
    End,
    /// "return"
    Return,
    /// "break"
    Break,
    /// "continue"
    Continue,
    /// "and"
    And,
    /// "or"
    Or,
    /// "not"
    Not,

    /// One character symbol.
    /// "+"
    Plus,
    /// "-"
    Minus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "%"
    Percent,
    /// "&"
    Amp,
    /// "|"
    Pipe,
    /// "^"
    Caret,
    /// "~"
    Tilde,
    /// "!"
    Bang,
    /// "="
    Eq,
    /// "<"
    Lt,
    /// ">"
    Gt,
    /// "."
    Dot,
    /// "@"
    At,
    /// ","
    Comma,
    /// ":"
    Colon,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,

    // Two character symbol.
    /// "->"
    Arrow,
    /// "!="
    BangEq,
    /// "=="
    Eq2,
    /// "<<"
    Lt2,
    /// "<="
    LtEq,
    /// ">>"
    Gt2,
    /// ">="
    GtEq,
    /// ".."
    Dot2,

    /// Identifier that is not classified as a keyword or literal. e.g. "foo"
    Ident,

    /// Like the `Ident`, but containing invalid unicode codepoints.
    InvalidIdent,

    /// Unknown character, not expected by the lexer.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NumBase {
    /// Binary integer literal that starts with "0b".
    Binary = 2,
    /// Octal integer literal that starts with "0o".
    Octal = 8,
    /// Decimal integer literal.
    Decimal = 10,
    /// Hexadecimal integer literal that starts with "0x".
    Hexadecimal = 16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuoteKind {
    /// Single quote string literal.
    Single,
    /// Double quote string literal.
    Double,
}
