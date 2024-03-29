use ecow::EcoString;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    // literals
    Int(i64),
    Float(f64),
    String(EcoString),
    Bool(bool),
    Nil,

    // keywords
    Var,
    Func,
    If,
    Then,
    Elif,
    Else,
    For,
    While,
    In,
    Ref,
    Do,
    End,
    Return,
    Break,
    Continue,

    // operators
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Mod,       // %
    Amp,       // &
    Pipe,      // |
    Caret,     // ^
    Tilde,     // ~
    Eq,        // ==
    NotEq,     // !=
    Less,      // <
    LessEq,    // <=
    Less2,     // <<
    Greater,   // >
    GreaterEq, // >=
    Greater2,  // >>
    Dot,       // .
    Arrow,     // ->
    Dot2,      // ..
    Assign,    // =

    // keyword operators
    And,
    Or,
    Not,

    // delimiters
    Comma,        // ,
    Colon,        // :
    OpenParen,    // (
    CloseParen,   // )
    OpenBrace,    // {
    CloseBrace,   // }
    OpenBracket,  // [
    CloseBracket, // ]

    // other
    Ident(&'src str),
    Attribute(&'src str),
    Comment(&'src str),
    Error(&'src str),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Int(x) => write!(f, "{}", x),
            Token::Float(x) => write!(f, "{}", x),
            Token::String(x) => write!(f, "\"{}\"", x),
            Token::Bool(x) => write!(f, "{}", if *x { "true" } else { "false" }),
            Token::Nil => write!(f, "nil"),
            Token::Var => write!(f, "var"),
            Token::Func => write!(f, "func"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Elif => write!(f, "elif"),
            Token::Else => write!(f, "else"),
            Token::For => write!(f, "for"),
            Token::While => write!(f, "while"),
            Token::In => write!(f, "in"),
            Token::Ref => write!(f, "ref"),
            Token::Do => write!(f, "do"),
            Token::End => write!(f, "end"),
            Token::Return => write!(f, "return"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Mod => write!(f, "%"),
            Token::Amp => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Caret => write!(f, "^"),
            Token::Tilde => write!(f, "~"),
            Token::Eq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEq => write!(f, "<="),
            Token::Less2 => write!(f, "<<"),
            Token::Greater => write!(f, ">"),
            Token::GreaterEq => write!(f, ">="),
            Token::Greater2 => write!(f, ">>"),
            Token::Dot => write!(f, "."),
            Token::Arrow => write!(f, "->"),
            Token::Dot2 => write!(f, ".."),
            Token::Assign => write!(f, "="),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::Ident(x) => write!(f, "{}", x),
            Token::Attribute(x) => write!(f, "@{}", x),
            Token::Comment(x) => write!(f, "#{}", x),
            Token::Error(c) => write!(f, "Error('{}')", c),
        }
    }
}
