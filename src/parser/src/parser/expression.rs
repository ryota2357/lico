use super::*;

impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
    // None = the next token cannot be parsed as an expression.
    pub fn expression(&mut self) -> Option<(Expression<'src>, TextSpan)> {
        let Some((token, _)) = self.look(0) else {
            return None;
        };

        match token {
            Token::Int(_) | Token::Float(_) | Token::String(_) | Token::Bool(_) | Token::Nil => {
                Some(self.expr_bp(0))
            }

            // keywords
            Token::Var => None,
            Token::Func => Some(self.expr_bp(0)),
            Token::If => None,
            Token::Then => None,
            Token::Elif => None,
            Token::Else => None,
            Token::For => None,
            Token::While => None,
            Token::In => None,
            Token::Ref => None,
            Token::Do => None,
            Token::End => None,
            Token::Return => None,
            Token::Break => None,
            Token::Continue => None,

            // operators
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Div
            | Token::Mod
            | Token::Eq
            | Token::NotEq
            | Token::Less
            | Token::LessEq
            | Token::Greater
            | Token::GreaterEq
            | Token::Dot
            | Token::Arrow
            | Token::Dot2
            | Token::Assign => Some(self.expr_bp(0)),

            // keyword operators
            Token::And | Token::Or | Token::Not => Some(self.expr_bp(0)),

            // delimiters
            Token::Comma => None,
            Token::Colon => None,
            Token::OpenParen => Some(self.expr_bp(0)),
            Token::CloseParen => None,
            Token::OpenBrace => Some(self.expr_bp(0)),
            Token::CloseBrace => None,
            Token::OpenBracket => Some(self.expr_bp(0)),
            Token::CloseBracket => None,

            // other
            Token::Ident(_) => Some(self.expr_bp(0)),
            Token::Attribute(_) => None,
            Token::Comment(_) => {
                loop {
                    self.move_next();
                    if !matches!(self.look(0), Some((Token::Comment(_), _))) {
                        break;
                    }
                }
                self.expression()
            }
            Token::Error(_) => todo!(),
        }
    }

    fn expr_bp(&mut self, min_bp: u8) -> (Expression<'src>, TextSpan) {
        let Some((current, current_span)) = self.next() else {
            let eoi_span = self.eoi_span();
            self.report(Error::UnexpectedEof("<expr>", eoi_span));
            return (Expression::Error, eoi_span);
        };

        let (mut lhs, mut lhs_span) = match binding_power::prefix_op(current) {
            Some((op, r_bp)) => {
                let (rhs, rhs_span) = self.expr_bp(r_bp);
                let span = TextSpan::new(current_span.start(), rhs_span.end());
                match (op, rhs) {
                    (UnaryOp::Neg, Expression::Primitive(Primitive::Int(x), _)) => {
                        (Expression::Primitive(Primitive::Int(-x), span), span)
                    }
                    (UnaryOp::Neg, Expression::Primitive(Primitive::Float(x), _)) => {
                        (Expression::Primitive(Primitive::Float(-x), span), span)
                    }
                    (op, rhs) => (
                        Expression::Unary {
                            op,
                            expr: (Box::new(rhs), rhs_span),
                        },
                        span,
                    ),
                }
            }
            None => match current {
                Token::Int(x) => (
                    Expression::Primitive(Primitive::Int(*x), current_span),
                    current_span,
                ),
                Token::Float(x) => (
                    Expression::Primitive(Primitive::Float(*x), current_span),
                    current_span,
                ),
                Token::String(x) => (
                    Expression::Primitive(Primitive::String(x.clone()), current_span),
                    current_span,
                ),
                Token::Bool(x) => (
                    Expression::Primitive(Primitive::Bool(*x), current_span),
                    current_span,
                ),
                Token::Nil => (
                    Expression::Primitive(Primitive::Nil, current_span),
                    current_span,
                ),
                Token::Ident(x) => (Expression::Local(x, current_span), current_span),
                Token::Func => {
                    let (args, _) = match self.look(0) {
                        Some((Token::OpenParen, _)) => {
                            self.move_next();
                            self.func_def_args()
                        }
                        Some(_) => todo!("implement error recovery"),
                        None => todo!("implement error recovery"),
                    };
                    let (body, end_span) = self.block_until_end_token();
                    (
                        Expression::FunctionObject(FunctionObject {
                            args,
                            body: Chunk {
                                captures: vec![],
                                block: body,
                            },
                        }),
                        TextSpan::new(current_span.start(), end_span.end()),
                    )
                }
                Token::OpenBrace => {
                    let (fields, close_span) = {
                        if let Some((Token::CloseBrace, span)) = self.look(0) {
                            let close_span = *span;
                            self.move_next();
                            (Vec::new(), close_span)
                        } else {
                            let mut fields = Vec::new();
                            let close_span = loop {
                                let field = match self.next() {
                                    Some((Token::Ident(name), name_span)) => {
                                        if let Some((Token::Assign, _)) = self.look(0) {
                                            self.move_next();
                                            let Some(expr) = self.expression() else {
                                                todo!("implement error recovery");
                                            };
                                            (TableFieldKey::Ident(name, name_span), expr)
                                        } else {
                                            todo!("implement error recovery");
                                        }
                                    }
                                    Some((Token::OpenBracket, _)) => {
                                        todo!("implement [expr] key")
                                    }
                                    // Some((Token::Func, func_span)) => {}
                                    Some(_) => todo!("implement error recovery"),
                                    None => todo!("implement error recovery"),
                                };
                                fields.push(field);
                                let Some((delim, delim_span)) = self.look(0) else {
                                    let eoi_span = self.eoi_span();
                                    self.report(Error::UnexpectedEof("}", eoi_span));
                                    break eoi_span;
                                };
                                match delim {
                                    Token::Comma => {
                                        self.move_next();
                                        if let Some((Token::CloseBrace, span)) = self.look(0) {
                                            let close_span = *span;
                                            self.move_next();
                                            break close_span;
                                        }
                                    }
                                    Token::CloseBrace => {
                                        let close_span = *delim_span;
                                        self.move_next();
                                        break close_span;
                                    }
                                    _ => todo!("implement error recovery"),
                                }
                            };
                            (fields, close_span)
                        }
                    };
                    (
                        Expression::TableObject(TableObject(fields)),
                        TextSpan::new(current_span.start(), close_span.end()),
                    )
                }
                Token::OpenBracket => {
                    let (exprs, close_span) = {
                        if let Some((Token::CloseBracket, span)) = self.look(0) {
                            let close_span = *span;
                            self.move_next();
                            (Vec::new(), close_span)
                        } else {
                            let mut exprs = Vec::new();
                            let close_span = loop {
                                let Some((expr, expr_span)) = self.expression() else {
                                    match self.look(0) {
                                        Some((Token::Comma, span)) => {
                                            self.report(Error::UnexpectedSymbol(",", *span));
                                            self.move_next();
                                            continue;
                                        }
                                        Some(_) => {
                                            todo!("implement error recovery");
                                        }
                                        None => {
                                            let eoi_span = self.eoi_span();
                                            self.report(Error::UnexpectedEof("]", eoi_span));
                                            break eoi_span;
                                        }
                                    }
                                };
                                exprs.push((expr, expr_span));
                                let Some((delim, delim_span)) = self.look(0) else {
                                    let eoi_span = self.eoi_span();
                                    self.report(Error::UnexpectedEof("]", eoi_span));
                                    break eoi_span;
                                };
                                match delim {
                                    Token::Comma => {
                                        self.move_next();
                                        if let Some((Token::CloseBracket, span)) = self.look(0) {
                                            let close_span = *span;
                                            self.move_next();
                                            break close_span;
                                        }
                                    }
                                    Token::CloseBracket => {
                                        let close_span = *delim_span;
                                        self.move_next();
                                        break close_span;
                                    }
                                    _ => todo!("implement error recovery"),
                                }
                            };
                            (exprs, close_span)
                        }
                    };
                    (
                        Expression::ArrayObject(ArrayObject(exprs)),
                        TextSpan::new(current_span.start(), close_span.end()),
                    )
                }
                Token::OpenParen => {
                    let (expr, _) = self.expr_bp(0);
                    let close_span = match self.next() {
                        Some((Token::CloseParen, close_span)) => close_span,
                        Some(_) => todo!("implement error recovery"),
                        None => {
                            let eoi_span = self.eoi_span();
                            self.report(Error::UnexpectedEof(")", eoi_span));
                            eoi_span
                        }
                    };
                    let span = TextSpan::new(current_span.start(), close_span.end());
                    (expr, span)
                }
                Token::Comment(_) => {
                    loop {
                        self.move_next();
                        if !matches!(self.look(0), Some((Token::Comment(_), _))) {
                            break;
                        }
                    }
                    return self.expr_bp(min_bp);
                }
                _ => {
                    let missing_expr_span = {
                        let prev_span = self
                            .look(-2) // If use -1, it will be the `current_span` because we already moved next by `self.next()`
                            .map(|(_, span)| *span)
                            .unwrap_or(TextSpan::new(0, 0));
                        TextSpan::new(prev_span.end(), current_span.start())
                    };
                    self.report(Error::MissingRequiredElement("<expr>", missing_expr_span));
                    // try recover to a binary expression.
                    if let Some((op, (l_bp, r_bp), err)) = binding_power::infix_op(current) {
                        if let Some(err) = err {
                            self.report(Error::Contextual(err, current_span));
                        }
                        if l_bp < min_bp {
                            self.move_prev();
                            return (Expression::Error, missing_expr_span);
                        }
                        let (rhs, rhs_span) = self.expr_bp(r_bp);
                        let span = TextSpan::new(current_span.start(), rhs_span.end());
                        (
                            Expression::Binary {
                                op,
                                lhs: (Box::new(Expression::Error), missing_expr_span),
                                rhs: (Box::new(rhs), rhs_span),
                            },
                            span,
                        )
                    } else {
                        self.move_prev();
                        return (Expression::Error, missing_expr_span);
                    }
                }
            },
        };

        loop {
            let Some((current, _)) = self.look(0) else {
                break;
            };

            if let Some(l_bp) = binding_power::postfix_op(current) {
                if l_bp < min_bp {
                    break;
                }
                let (current, _) = unsafe { self.next().unwrap_unchecked() }; // SAFETY: this is checked to be Some() by above self.look(0)
                (lhs, lhs_span) = match current {
                    Token::OpenParen => {
                        let (args, close_span) = self.func_call_args();
                        (
                            Expression::Call {
                                expr: (Box::new(lhs), lhs_span),
                                args,
                            },
                            TextSpan::new(lhs_span.start(), close_span.end()),
                        )
                    },
                    Token::Arrow => {
                        let (name, name_span) = match self.next() {
                            Some((Token::Ident(x), span)) => (*x, span),
                            _ => {
                                self.move_prev();
                                todo!("implement error recovery")
                            }
                        };
                        match self.next() {
                            Some((Token::OpenParen, _)) => {},
                            _ => todo!("implement error recovery"),
                        }
                        let (args, close_span) = self.func_call_args();
                        (
                            Expression::MethodCall {
                                expr: (Box::new(lhs), lhs_span),
                                name: (name, name_span),
                                args,
                            },
                            TextSpan::new(lhs_span.start(), close_span.end()),
                        )
                    },
                    Token::Dot => {
                        let (name, name_span) = match self.next() {
                            Some((Token::Ident(x), span)) => (*x, span),
                            _ => {
                                self.move_prev();
                                todo!("implement error recovery")
                            }
                        };
                        (
                            Expression::DotAccess {
                                expr: (Box::new(lhs), lhs_span),
                                accessor: (name, name_span),
                            },
                            TextSpan::new(lhs_span.start(), name_span.end()),
                        )
                    },
                    Token::OpenBracket => {
                        let Some((expr, expr_span)) = self.expression() else {
                            todo!("implement error recovery");
                        };
                        if let Some((Token::CloseBracket, close_span)) = self.next() {
                            (
                                Expression::IndexAccess {
                                    expr: (Box::new(lhs), lhs_span),
                                    accessor: (Box::new(expr), expr_span)
                                },
                                TextSpan::new(lhs_span.start(), close_span.end()),
                            )
                        } else {
                            self.move_prev();
                            todo!("implement error recovery");
                        }
                    },
                    _ => unreachable!(
                        "binding_power::postfix_op() should only return Some() for valid postfix operators"
                    ),
                };
                continue;
            };

            if let Some((op, (l_bp, r_bp), err)) = binding_power::infix_op(current) {
                if let Some(err) = err {
                    self.report(Error::Contextual(err, current_span));
                }
                if l_bp < min_bp {
                    break;
                }
                self.move_next();
                let (rhs, rhs_span) = self.expr_bp(r_bp);
                (lhs, lhs_span) = (
                    Expression::Binary {
                        op,
                        lhs: (Box::new(lhs), lhs_span),
                        rhs: (Box::new(rhs), rhs_span),
                    },
                    TextSpan::new(lhs_span.start(), rhs_span.end()),
                );
                continue;
            };

            break;
        }
        (lhs, lhs_span)
    }
}

/// |       Precedence        | Associativity |   Operators   |
/// | ----------------------- | ------------- | ------------- |
/// | 9: Call                 |    postfix    | (), ->ident() |
/// | 8: Dot and Indexing     |    postfix    | .ident, []    |
/// | 7: Unary                |    prefix     | +, -, not     |
/// | 6: Multiplicative       |   left infix  | *, /, %       |
/// | 5: Additive             |   left infix  | +, -          |
/// | 4: String concatenation |  right infix  | ..            |
/// | 3: Relational           |   left infix  | <, <=, >, >=  |
/// | 2: Equality             |   left infix  | ==, !=        |
/// | 1: Logical-AND          |   left infix  | and           |
/// | 0: Logical-OR           |   left infix  | or            |
mod binding_power {
    use super::*;

    const CALL: u8 = 9;
    const DOT_INDEX: u8 = 8;
    const UNARY: u8 = 7;
    const MULTIPLICATIVE: u8 = 6;
    const ADDITIVE: u8 = 5;
    const STRING_CONCAT: u8 = 4;
    const RELATIONAL: u8 = 3;
    const EQUALITY: u8 = 2;
    const LOGICAL_AND: u8 = 1;
    const LOGICAL_OR: u8 = 0;

    pub fn prefix_op(token: &Token) -> Option<(UnaryOp, u8)> {
        #[rustfmt::skip]
        let op = match token {
            Token::Minus => UnaryOp::Neg,
            Token::Not   => UnaryOp::Not,
            _ => return None,
        };
        Some((op, 2 * UNARY + 1))
    }

    pub fn postfix_op(token: &Token) -> Option<u8> {
        #[rustfmt::skip]
        let precedence = match token {
            Token::OpenParen   => CALL,
            Token::Arrow       => CALL,
            Token::Dot         => DOT_INDEX,
            Token::OpenBracket => DOT_INDEX,
            _ => return None,
        };
        Some(2 * precedence + 2)
    }

    pub fn infix_op(token: &Token) -> Option<(BinaryOp, (u8, u8), Option<String>)> {
        const fn left(precedence: u8) -> (u8, u8) {
            (2 * precedence + 1, 2 * precedence + 2)
        }
        const fn right(precedence: u8) -> (u8, u8) {
            (2 * precedence + 2, 2 * precedence + 1)
        }

        #[rustfmt::skip]
        let (bp, err, op) = match token {
            Token::Star      => (left(MULTIPLICATIVE), None, BinaryOp::Mul),
            Token::Div       => (left(MULTIPLICATIVE), None, BinaryOp::Div),
            Token::Mod       => (left(MULTIPLICATIVE), None, BinaryOp::Mod),
            Token::Plus      => (left(ADDITIVE),       None, BinaryOp::Add),
            Token::Minus     => (left(ADDITIVE),       None, BinaryOp::Sub),
            Token::Dot2      => (right(STRING_CONCAT), None, BinaryOp::Concat),
            Token::Less      => (left(RELATIONAL),     None, BinaryOp::Less),
            Token::LessEq    => (left(RELATIONAL),     None, BinaryOp::LessEq),
            Token::Greater   => (left(RELATIONAL),     None, BinaryOp::Greater),
            Token::GreaterEq => (left(RELATIONAL),     None, BinaryOp::GreaterEq),
            Token::Eq        => (left(EQUALITY),       None, BinaryOp::Eq),
            Token::NotEq     => (left(EQUALITY),       None, BinaryOp::NotEq),
            Token::And       => (left(LOGICAL_AND),    None, BinaryOp::And),
            Token::Or        => (left(LOGICAL_OR),     None, BinaryOp::Or),
            // Token::Assign    => {
            //     let err = "Should use `==` for equal".to_string();
            //     (left(EQUALITY), Some(err), BinaryOp::Eq)
            // }
            Token::Error("~=") => {
                let err = "Should use `!=` for not equal".to_string();
                (left(EQUALITY), Some(err), BinaryOp::NotEq)
            }
            _ => return None,
        };
        Some((op, bp, err))
    }
}
