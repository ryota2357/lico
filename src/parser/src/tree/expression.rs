use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'src> {
    Unary {
        op: UnaryOp,
        expr: Box<Expression<'src>>,
    },
    Binary {
        op: BinaryOp,
        rhs: Box<Expression<'src>>,
        lhs: Box<Expression<'src>>,
    },
    Primitive(Primitive<'src>),
    TableObject(TableObject<'src>),
    ArrayObject(ArrayObject<'src>),
    FunctionObject(FunctionObject<'src>),
    Call(Call<'src>),
    Local(Local<'src>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg, // -
    Not, // not
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Pow, // *

    // table access
    Dot, // .

    // comparison
    Eq,        // ==
    NotEq,     // !=
    Less,      // <
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=

    // logical
    And, // and
    Or,  // or
}

/// <Expression> ::= '(' <Expression> ')' | <expression>
/// <expression> ::= <Unary> | <Binary> | <Primitive> | <TableObject> | <ArrayObject> | <FunctionObject> | <Call> | <Local>
pub(super) fn expression<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>> + Clone
{
    let expr = recursive(|expr| {
        // TODO: unary と binary は pratt パーサー (ver2) がマージされたら
        let primitive = primitive().map(Expression::Primitive);
        let table_object = table_object(expr.clone()).map(Expression::TableObject);
        let array_object = array_object(expr.clone()).map(Expression::ArrayObject);
        let function_object = function_object(block.clone()).map(Expression::FunctionObject);
        let call = call(block, expr).map(Expression::Call);
        let local = local().map(Expression::Local);

        choice((
            primitive,
            table_object,
            array_object,
            function_object,
            call,
            local,
        ))
    });

    let delimited_expr = expr
        .clone()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

    delimited_expr.or(expr)
}

impl<'a> TreeWalker<'a> for Expression<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Expression::Unary { expr, .. } => expr.analyze(tracker),
            Expression::Binary { rhs, lhs, .. } => {
                rhs.analyze(tracker);
                lhs.analyze(tracker);
            }
            Expression::Primitive(_) => {}
            Expression::TableObject(table_object) => table_object.analyze(tracker),
            Expression::ArrayObject(array_object) => array_object.analyze(tracker),
            Expression::FunctionObject(function_object) => function_object.analyze(tracker),
            Expression::Call(call) => call.analyze(tracker),
            Expression::Local(local) => match local {
                Local::TableField { name, .. } => tracker.add_capture(name.str),
                Local::Variable { name } => tracker.add_capture(name.str),
            },
        }
    }
}
