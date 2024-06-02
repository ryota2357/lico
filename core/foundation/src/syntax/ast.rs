use super::{LicoLanguage, SyntaxKind, SyntaxNode, SyntaxToken, T};
use rowan::ast::{support, AstChildren, AstNode};

macro_rules! __node_method {
    (child, $name:ident, $type:ty) => {
        pub fn $name(&self) -> Option<$type> {
            support::child(AstNode::syntax(self))
        }
    };
    (child, $name:ident, $type:ty, $idx:literal) => {
        pub fn $name(&self) -> Option<$type> {
            support::children(AstNode::syntax(self)).nth($idx)
        }
    };
    (children, $name:ident, $type:ty) => {
        pub fn $name(&self) -> AstChildren<$type> {
            support::children(AstNode::syntax(self))
        }
    };
    (token, $name:ident, $kind:tt) => {
        pub fn $name(&self) -> Option<SyntaxToken> {
            support::token(AstNode::syntax(self), T![$kind])
        }
    };
}
macro_rules! ast_node {
    (struct $name:ident for $kind:ident $({
        $( $method:ident: $method_kind:ident[$method_ret:tt $(, $method_ret_info:tt)?] ),* $(,)?
    })?) => {
        pub struct $name(SyntaxNode);
        impl AstNode for $name {
            type Language = LicoLanguage;
            fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool {
                kind == SyntaxKind::$kind
            }
            fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self> {
                if Self::can_cast(node.kind()) {
                    Some(Self(node))
                } else {
                    None
                }
            }
            fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
                &self.0
            }
        }
        $(
            impl $name {
                $( __node_method! { $method_kind, $method, $method_ret $(, $method_ret_info)? })*
            }
        )?
    };
    (enum $name:ident for {
        $( $variant:ident($node:ty) ),* $(,)?
    }) => {
        pub enum $name {
            $($variant($node)),*
        }
        impl AstNode for $name {
            type Language = LicoLanguage;
            fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool {
                $(<$node as AstNode>::can_cast(kind))||*
            }
            fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self> {
                let kind = node.kind();
                $(
                    if <$node as AstNode>::can_cast(kind) {
                        let casted = <$node as AstNode>::cast(node).expect("Invalid `can_cast` implementation");
                        return Some(Self::$variant(casted));
                    }
                )*
                None
            }
            fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
                match self {
                    $(Self::$variant(x) => x.syntax()),*
                }
            }
        }
    }
}

// Statement*
ast_node!(struct Program for PROGRAM {
    statements: children[Statement],
});

ast_node!(enum Statement for {
    Var(VarStmt),
    Func(FuncStmt),
    For(ForStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Expr(ExprStmt),
    Attr(AttrStmt),
});

// 'var' Name '=' Expr
ast_node!(struct VarStmt for VAR_STMT {
    var_token: token[var],
    name: child[Name],
    eq_token: token[=],
    initializer: child[Expression],
});

// 'func' NamePath ('->' Name)? ParamList
//   Program
// 'end'
ast_node!(struct FuncStmt for FUNC_STMT {
    func_token: token[func],
    name_path: child[NamePath],
    arrow_token: token[->],
    method_name: child[Name],
    param_list: child[ParamList],
    body: child[Program],
    end_token: token[end],
});

// 'for' Name 'in' Expr 'do'
//   Program
// 'end'
ast_node!(struct ForStmt for FOR_STMT {
    for_token: token[for],
    name: child[Name],
    in_token: token[in],
    iterable: child[Expression],
    do_token: token[do],
    body: child[Program],
    end_token: token[end],
});

// 'while' Expr 'do'
//   Program
// 'end'
ast_node!(struct WhileStmt for WHILE_STMT {
    while_token: token[while],
    condition: child[Expression],
    do_token: token[do],
    body: child[Program],
    end_token: token[end],
});

// 'return' Expr?
ast_node!(struct ReturnStmt for RETURN_STMT {
    return_token: token[return],
    expr: child[Expression],
});

// 'break'
ast_node!(struct BreakStmt for BREAK_STMT {
    break_token: token[break],
});

// 'continue'
ast_node!(struct ContinueStmt for CONTINUE_STMT {
    continue_token: token[continue],
});

// Expr
ast_node!(struct ExprStmt for EXPR_STMT {
    expr: child[Expression],
});

// '@' Name
ast_node!(struct AttrStmt for ATTR_STMT {
    at_token: token[@],
    name: child[Name],
});

ast_node!(enum Expression for {
    If(IfExpr),
    Do(DoExpr),
    Call(CallExpr),
    Binary(BinaryExpr),
    Prefix(PrefixExpr),
    Index(IndexExpr),
    Field(FieldExpr),
    MethodCall(MethodCallExpr),
    Paren(ParenExpr),
    LocalVar(LocalVar),
    Literal(Literal),
    ArrayConst(ArrayConst),
    TableConst(TableConst),
    FuncConst(FuncConst),
});

// 'if' Expr then
//   Program
//  ElifBranch*
//  ElseBranch?
// 'end'
ast_node!(struct IfExpr for IF_EXPR {
    if_token: token[if],
    condition: child[Expression],
    then_token: token[then],
    body: child[Program],
    elif_branches: children[ElifBranch],
    else_branch: child[ElseBranch],
    end_token: token[end],
});

// 'do'
//   Program
// 'end'
ast_node!(struct DoExpr for DO_EXPR {
    do_token: token[do],
    body: child[Program],
    end_token: token[end],
});

// Expr ArgList
ast_node!(struct CallExpr for CALL_EXPR {
    expr: child[Expression],
    arg_list: child[ArgList],
});

// Expr 'BinaryOp' Expr
ast_node!(struct BinaryExpr for BINARY_EXPR {
    lhs: child[Expression],
    rhs: child[Expression],
});
impl BinaryExpr {
    pub fn op(&self) -> Option<(SyntaxToken, BinaryOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(|token| {
                #[rustfmt::skip]
                let op = match token.kind() {
                    T![+]   => BinaryOp::Add,
                    T![-]   => BinaryOp::Sub,
                    T![*]   => BinaryOp::Mul,
                    T![/]   => BinaryOp::Div,
                    T![%]   => BinaryOp::Mod,
                    T![<<]  => BinaryOp::Shl,
                    T![>>]  => BinaryOp::Shr,
                    T![..]  => BinaryOp::Concat,
                    T![==]  => BinaryOp::Eq,
                    T![!=]  => BinaryOp::Ne,
                    T![<]   => BinaryOp::Lt,
                    T![<=]  => BinaryOp::Le,
                    T![>]   => BinaryOp::Gt,
                    T![>=]  => BinaryOp::Ge,
                    T![and] => BinaryOp::And,
                    T![or]  => BinaryOp::Or,
                    T![&]   => BinaryOp::BitAnd,
                    T![|]   => BinaryOp::BitOr,
                    T![^]   => BinaryOp::BitXor,
                    T![=]   => BinaryOp::Assign,
                    _ => return None,
                };
                Some((token, op))
            })
    }
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op().map(|(token, _)| token)
    }
    pub fn op_kind(&self) -> Option<BinaryOp> {
        self.op().map(|(_, kind)| kind)
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    Concat,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Assign,
}

ast_node!(struct PrefixExpr for PREFIX_EXPR {
    expr: child[Expression],
});
impl PrefixExpr {
    pub fn op(&self) -> Option<(SyntaxToken, PrefixOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(|token| {
                #[rustfmt::skip]
                let op = match token.kind() {
                    T![+]   => PrefixOp::Plus,
                    T![-]   => PrefixOp::Minus,
                    T![not] => PrefixOp::Not,
                    T![~]   => PrefixOp::BitNot,
                    _ => return None,
                };
                Some((token, op))
            })
    }
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op().map(|(token, _)| token)
    }
    pub fn op_kind(&self) -> Option<PrefixOp> {
        self.op().map(|(_, kind)| kind)
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrefixOp {
    Plus,
    Minus,
    Not,
    BitNot,
}

// Expr '[' Expr ']'
ast_node!(struct IndexExpr for INDEX_EXPR {
    expr: child[Expression, 0],
    l_bracket_token: token['['],
    index: child[Expression, 1],
    r_bracket_token: token[']'],
});

// Expr '.' Name
ast_node!(struct FieldExpr for FIELD_EXPR {
    expr: child[Expression],
    dot_token: token[.],
    field: child[Name],
});

// Expr '->' Name ArgList
ast_node!(struct MethodCallExpr for METHOD_CALL_EXPR {
    expr: child[Expression],
    arrow_token: token[->],
    method_name: child[Name],
    arg_list: child[ArgList],
});

// '(' Expr ')'
ast_node!(struct ParenExpr for PAREN_EXPR {
    l_paren_token: token['('],
    expr: child[Expression],
    r_paren_token: token[')'],
});

// ident
ast_node!(struct LocalVar for LOCAL_VAR {
    ident_token: token[ident],
});

// int | float | string | 'true' | 'false' | nil
ast_node!(struct Literal for LITERAL);
impl Literal {
    pub fn token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| it.kind().is_literal())
    }
    pub fn kind(&self) -> Option<LiteralKind> {
        let token = self.token()?;
        #[rustfmt::skip]
        let kind = match token.kind() {
            T![int]    => LiteralKind::Int(token),
            T![float]  => LiteralKind::Float(token),
            T![string] => LiteralKind::String(token),
            T![true]   => LiteralKind::Bool(true),
            T![false]  => LiteralKind::Bool(false),
            T![nil]    => LiteralKind::Nil,
            _ => return None,
        };
        Some(kind)
    }
}
pub enum LiteralKind {
    Int(SyntaxToken),
    Float(SyntaxToken),
    String(SyntaxToken),
    Bool(bool),
    Nil,
}

// '[' ( Expr (',' Expr)* ','? )? ']'
ast_node!(struct ArrayConst for ARRAY_CONST {
    l_bracket_token: token['['],
    elements: children[Expression],
    r_bracket_token: token[']'],
});

// '[' TableField* ']'
ast_node!(struct TableConst for TABLE_CONST {
    l_brace_token: token['{'],
    fields: children[TableField],
    r_brace_token: token['}'],
});

// 'func' ParamList
//   Program
// 'end'
ast_node!(struct FuncConst for FUNC_CONST {
    func_token: token[func],
    param_list: child[ParamList],
    body: child[Program],
    end_token: token[end],
});

// 'else'
//   Program
ast_node!(struct ElseBranch for ELSE_BRANCH {
    else_token: token[else],
    body: child[Program],
});

// 'elif' Expr 'then'
//   Program
ast_node!(struct ElifBranch for ELIF_BRANCH {
    elif_token: token[elif],
    condition: child[Expression],
    then_token: token[then],
    body: child[Program],
});

// '(' Name* ')'
ast_node!(struct ParamList for PARAM_LIST {
    l_paren_token: token['('],
    params: children[Name],
    r_paren_token: token[')'],
});

// '(' Expr* ')'
ast_node!(struct ArgList for ARG_LIST {
    l_paren_token: token['('],
    args: children[Expression],
    r_paren_token: token[')'],
});

// ident
ast_node!(struct Name for NAME {
    ident_token: token[ident],
});

// Name (. NamePath)?
ast_node!(struct NamePath for NAME_PATH {
    name: child[Name],
    dot_token: token[.],
    child: child[NamePath],
});

// 'func'? TableFieldName = Expr
ast_node!(struct TableField for TABLE_FIELD {
    func_token: token[func],
    field_name: child[TableFieldName],
    eq_token: token[=],
    initializer: child[Expression],
});

// TableFieldNameIdent | TableFieldNameExpr
ast_node!(enum TableFieldName for {
    Ident(TableFieldNameIdent),
    Expr(TableFieldNameExpr),
});

// '[' Expr ']'
ast_node!(struct TableFieldNameExpr for TABLE_FIELD_NAME_EXPR {
    l_bracket_token: token['['],
    expr: child[Expression],
    r_bracket_token: token[']'],
});

// ident
ast_node!(struct TableFieldNameIdent for TABLE_FIELD_NAME_IDENT {
    ident_token: token[ident],
});
