pub mod ast;
pub mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LicoLanguage {}

impl rowan::Language for LicoLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from(raw)
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<LicoLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<LicoLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<LicoLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<LicoLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<LicoLanguage>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<LicoLanguage>;

macro_rules! syntax_kind {
    ($($variant:ident $(= [$($tt:tt)*])? $(@ $anchor:ident)?),* $(,)?) => {
        #[allow(bad_style)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(u8)]
        pub enum SyntaxKind {
            $(
                $variant
            ),*
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! __token_kind_fast_accsess {
            $($(
                ($($tt)*) => { $crate::syntax::SyntaxKind::$variant };
            )?)*
        }
        pub use __token_kind_fast_accsess as T;

        impl From<SyntaxKind> for rowan::SyntaxKind {
            fn from(kind: SyntaxKind) -> Self {
                Self(kind as u16)
            }
        }

        impl From<rowan::SyntaxKind> for SyntaxKind {
            fn from(kind: rowan::SyntaxKind) -> Self {
                let n_variant = $crate::macros::count!($($variant)*) as u16;
                assert!(kind.0 <= n_variant, "bad SyntaxKind: {:?}", kind);
                // SAFETY: Ensured by the assert above.
                unsafe { ::core::mem::transmute(kind.0 as u8) }
            }
        }

        impl SyntaxKind {
            $($(const $anchor: Self = Self::$variant;)?)*
        }
    };
}
syntax_kind! {
    ERROR,

    // ↓ Leaf (token) ↓

    COMMENT,
    WHITESPACE,

    INT = [int] @START_LITERAL,
    FLOAT = [float],
    STRING = [string],
    TRUE = [true],
    FALSE = [false],
    NIL = [nil] @END_LITERAL,

    VAR_KW = [var] @START_KEYWORD,
    FUNC_KW = [func],
    IF_KW = [if],
    THEN_KW = [then],
    ELIF_KW = [elif],
    ELSE_KW = [else],
    FOR_KW = [for],
    WHILE_KW = [while],
    IN_KW = [in],
    DO_KW = [do],
    END_KW = [end],
    RETURN_KW = [return],
    BREAK_KW = [break],
    CONTINUE_KW = [continue],
    AND_KW = [and],
    OR_KW = [or],
    NOT_KW = [not] @END_KEYWORD,

    PLUS = [+] @START_PUNCT,
    MINUS = [-],
    STAR = [*],
    SLASH = [/],
    PERCENT = [%],
    AMP = [&],
    PIPE = [|],
    CARET = [^],
    TILDE = [~],
    BANG = [!],
    EQ = [=],
    LT = [<],
    GT = [>],
    DOT = [.],
    AT = [@],
    COMMA = [,],
    COLON = [:],
    OPENPAREN = ['('],
    CLOSEPAREN = [')'],
    OPENBRACE = ['{'],
    CLOSEBRACE = ['}'],
    OPENBRACKET = ['['],
    CLOSEBRACKET = [']'],
    ARROW = [->],
    BANGEQ = [!=],
    EQ2 = [==],
    LT2 = [<<],
    LTEQ = [<=],
    GT2 = [>>],
    GTEQ = [>=],
    DOT2 = [..] @END_PUNCT,

    IDENT = [ident],

    // ↓ Node (non-leaf) ↓

    PROGRAM,

    VAR_STMT,
    FUNC_STMT,
    FOR_STMT,
    WHILE_STMT,
    RETURN_STMT,
    BREAK_STMT,
    CONTINUE_STMT,
    EXPR_STMT,
    ATTR_STMT,

    IF_EXPR,
    DO_EXPR,
    CALL_EXPR,
    BINARY_EXPR,
    PREFIX_EXPR,
    INDEX_EXPR,
    FIELD_EXPR,
    METHOD_CALL_EXPR,
    PAREN_EXPR,

    LOCAL_VAR,
    LITERAL,
    ARRAY_CONST,
    TABLE_CONST,
    FUNC_CONST,

    ELSE_BRANCH,
    ELIF_BRANCH,

    PARAM_LIST,
    ARG_LIST,

    NAME,
    NAME_PATH,
    TABLE_FIELD,
    TABLE_FIELD_NAME_IDENT,
    TABLE_FIELD_NAME_EXPR,
}

const fn _static_assert_size() {
    const { assert!(core::mem::size_of::<SyntaxKind>() == 1) }
}

impl SyntaxKind {
    fn is_between(self, start: SyntaxKind, end: SyntaxKind) -> bool {
        (start as u8..=end as u8).contains(&(self as u8))
    }
    pub fn is_literal(self) -> bool {
        self.is_between(SyntaxKind::START_LITERAL, SyntaxKind::END_LITERAL)
    }
    pub fn is_keyword(self) -> bool {
        self.is_between(SyntaxKind::START_KEYWORD, SyntaxKind::END_KEYWORD)
    }
    pub fn is_punct(self) -> bool {
        self.is_between(SyntaxKind::START_PUNCT, SyntaxKind::END_PUNCT)
    }
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
    }
}
