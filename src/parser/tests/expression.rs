mod common;
use parser::tree::*;

fn do_expr_test(src: &str, expression: Expression<'_>) {
    let src = format!("_={}", src);
    let program = common::parse_program(&src);
    let stats = program.body.body;
    assert_eq!(stats.len(), 1);
    let statement = &stats[0];
    if let Statement::Variable(VariableStatement::Assign { lhs: _, rhs }) = statement {
        assert_eq!(rhs, &expression);
    } else {
        panic!(
            "Expected VariableStatement::Assign, but got {:?}",
            statement
        );
    }
}

#[test]
fn delimited_call() {
    do_expr_test(
        "(f())",
        Expression::Invoke {
            expr: Box::new(Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (3..4).into(),
            }))),
            args: vec![],
        },
    );
    do_expr_test(
        "(((f(1))))",
        Expression::Invoke {
            expr: Box::new(Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (5..6).into(),
            }))),
            args: vec![Expression::Primitive(Primitive::Int(1))],
        },
    );
}

#[test]
fn delimited_pratt() {
    do_expr_test(
        "(1+2)",
        Expression::Binary {
            op: BinaryOp::Add,
            lhs: Box::new(Expression::Primitive(Primitive::Int(1))),
            rhs: Box::new(Expression::Primitive(Primitive::Int(2))),
        },
    );
    do_expr_test(
        "(((1+2*3)))",
        Expression::Binary {
            op: BinaryOp::Add,
            lhs: Box::new(Expression::Primitive(Primitive::Int(1))),
            rhs: Box::new(Expression::Binary {
                op: BinaryOp::Mul,
                lhs: Box::new(Expression::Primitive(Primitive::Int(2))),
                rhs: Box::new(Expression::Primitive(Primitive::Int(3))),
            }),
        },
    );
}

#[test]
fn delimited_primitive() {
    do_expr_test("(1)", Expression::Primitive(Primitive::Int(1)));
    do_expr_test(
        "((('abc')))",
        Expression::Primitive(Primitive::String("abc")),
    );
}

#[test]
fn delimited_local() {
    do_expr_test(
        "(a)",
        Expression::Local(Local::Ident(Ident {
            str: "a",
            span: (3..4).into(),
        })),
    );
    do_expr_test(
        "(((a)))",
        Expression::Local(Local::Ident(Ident {
            str: "a",
            span: (5..6).into(),
        })),
    );
}
#[test]
fn delimited_local_access() {
    do_expr_test(
        "((a.b).c)",
        Expression::Binary {
            op: BinaryOp::Dot,
            lhs: Box::new(Expression::Local(Local::Access {
                ident: Ident {
                    str: "a",
                    span: (4..5).into(),
                },
                keys: vec![Expression::Local(Local::Ident(Ident {
                    str: "b",
                    span: (6..7).into(),
                }))],
            })),
            rhs: Box::new(Expression::Local(Local::Ident(Ident {
                str: "c",
                span: (9..10).into(),
            }))),
        },
    );
    do_expr_test(
        "(a['b'])",
        Expression::Local(Local::Access {
            ident: Ident {
                str: "a",
                span: (3..4).into(),
            },
            keys: vec![Expression::Primitive(Primitive::String("b"))],
        }),
    );
}

#[test]
fn composition_func() {
    let expr = Expression::Invoke {
        expr: Box::new(Expression::Local(Local::Ident(Ident {
            str: "f",
            span: (2..3).into(),
        }))),
        args: vec![Expression::Invoke {
            expr: Box::new(Expression::Local(Local::Ident(Ident {
                str: "g",
                span: (4..5).into(),
            }))),
            args: vec![],
        }],
    };
    do_expr_test("f(g())", expr.clone());
    do_expr_test("f(g(),)", expr);
}

#[test]
fn multiple_call() {
    do_expr_test(
        "f()()",
        Expression::Invoke {
            expr: Box::new(Expression::Invoke {
                expr: Box::new(Expression::Local(Local::Ident(Ident {
                    str: "f",
                    span: (2..3).into(),
                }))),
                args: vec![],
            }),
            args: vec![],
        },
    );
    do_expr_test(
        "(f(1)(2))()(false)",
        Expression::Invoke {
            expr: Box::new(Expression::Invoke {
                expr: Box::new(Expression::Invoke {
                    expr: Box::new(Expression::Invoke {
                        expr: Box::new(Expression::Local(Local::Ident(Ident {
                            str: "f",
                            span: (3..4).into(),
                        }))),
                        args: vec![Expression::Primitive(Primitive::Int(1))],
                    }),
                    args: vec![Expression::Primitive(Primitive::Int(2))],
                }),
                args: vec![],
            }),
            args: vec![Expression::Primitive(Primitive::Bool(false))],
        },
    );
}

#[test]
#[should_panic]
fn error_func_call() {
    do_expr_test(
        "f(,)",
        Expression::Invoke {
            expr: Box::new(Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (8..9).into(),
            }))),
            args: vec![],
        },
    );
}

#[test]
fn arithmetic_op() {
    do_expr_test(
        "-1 + 2 * 3 / 4 - 5", // ((-1) + ((2 * 3) / 4)) - 5
        Expression::Binary {
            op: BinaryOp::Sub,
            lhs: Box::new(Expression::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(Expression::Primitive(Primitive::Int(-1))),
                rhs: Box::new(Expression::Binary {
                    op: BinaryOp::Div,
                    lhs: Box::new(Expression::Binary {
                        op: BinaryOp::Mul,
                        lhs: Box::new(Expression::Primitive(Primitive::Int(2))),
                        rhs: Box::new(Expression::Primitive(Primitive::Int(3))),
                    }),
                    rhs: Box::new(Expression::Primitive(Primitive::Int(4))),
                }),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Int(5))),
        },
    );
    do_expr_test(
        "1 * 2 ** 3 ** 4 / 5", // (1 * (2 ** (3 ** 4))) / 5
        Expression::Binary {
            op: BinaryOp::Div,
            lhs: Box::new(Expression::Binary {
                op: BinaryOp::Mul,
                lhs: Box::new(Expression::Primitive(Primitive::Int(1))),
                rhs: Box::new(Expression::Binary {
                    op: BinaryOp::Pow,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(2))),
                    rhs: Box::new(Expression::Binary {
                        op: BinaryOp::Pow,
                        lhs: Box::new(Expression::Primitive(Primitive::Int(3))),
                        rhs: Box::new(Expression::Primitive(Primitive::Int(4))),
                    }),
                }),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Int(5))),
        },
    );
}

#[test]
fn arithmetic_op_with_paren() {
    do_expr_test(
        "(-1 + 2) * 3",
        Expression::Binary {
            op: BinaryOp::Mul,
            lhs: Box::new(Expression::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(Expression::Primitive(Primitive::Int(-1))),
                rhs: Box::new(Expression::Primitive(Primitive::Int(2))),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Int(3))),
        },
    );
    do_expr_test(
        "(1 + (2 ** 3)) ** 4",
        Expression::Binary {
            op: BinaryOp::Pow,
            lhs: Box::new(Expression::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(Expression::Primitive(Primitive::Int(1))),
                rhs: Box::new(Expression::Binary {
                    op: BinaryOp::Pow,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(2))),
                    rhs: Box::new(Expression::Primitive(Primitive::Int(3))),
                }),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Int(4))),
        },
    )
}

#[test]
fn logical_op() {
    do_expr_test(
        "not a == 10 or 5 >= b and false", // ((not a) == 10) or ((5 >= b) and false)
        Expression::Binary {
            op: BinaryOp::Or,
            lhs: Box::new(Expression::Binary {
                op: BinaryOp::Eq,
                lhs: Box::new(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expression::Local(Local::Ident(Ident {
                        str: "a",
                        span: (6..7).into(),
                    }))),
                }),
                rhs: Box::new(Expression::Primitive(Primitive::Int(10))),
            }),
            rhs: Box::new(Expression::Binary {
                op: BinaryOp::And,
                lhs: Box::new(Expression::Binary {
                    op: BinaryOp::GreaterEq,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(5))),
                    rhs: Box::new(Expression::Local(Local::Ident(Ident {
                        str: "b",
                        span: (22..23).into(),
                    }))),
                }),
                rhs: Box::new(Expression::Primitive(Primitive::Bool(false))),
            }),
        },
    );
}

#[test]
fn logical_op_with_paren() {
    do_expr_test(
        "(not (a != 10) or 5 < b) and false",
        Expression::Binary {
            op: BinaryOp::And,
            lhs: Box::new(Expression::Binary {
                op: BinaryOp::Or,
                lhs: Box::new(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expression::Binary {
                        op: BinaryOp::NotEq,
                        lhs: Box::new(Expression::Local(Local::Ident(Ident {
                            str: "a",
                            span: (8..9).into(),
                        }))),
                        rhs: Box::new(Expression::Primitive(Primitive::Int(10))),
                    }),
                }),
                rhs: Box::new(Expression::Binary {
                    op: BinaryOp::Less,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(5))),
                    rhs: Box::new(Expression::Local(Local::Ident(Ident {
                        str: "b",
                        span: (24..25).into(),
                    }))),
                }),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Bool(false))),
        },
    )
}

