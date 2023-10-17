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
            expr: Box::new(Expression::Ident(Ident {
                str: "f",
                span: (3..4).into(),
            })),
            args: vec![],
        },
    );
    do_expr_test(
        "(((f(1))))",
        Expression::Invoke {
            expr: Box::new(Expression::Ident(Ident {
                str: "f",
                span: (5..6).into(),
            })),
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
        Expression::Ident(Ident {
            str: "a",
            span: (3..4).into(),
        }),
    );
    do_expr_test(
        "(((a)))",
        Expression::Ident(Ident {
            str: "a",
            span: (5..6).into(),
        }),
    );
}
#[test]
fn delimited_local_access() {
    do_expr_test(
        "((a.b).c)",
        Expression::DotAccess {
            expr: Box::new(Expression::DotAccess {
                expr: Box::new(Expression::Ident(Ident {
                    str: "a",
                    span: (4..5).into(),
                })),
                accesser: Ident {
                    str: "b",
                    span: (6..7).into(),
                },
            }),
            accesser: Ident {
                str: "c",
                span: (9..10).into(),
            },
        },
    );
    do_expr_test(
        "(a['b'])",
        Expression::IndexAccess {
            expr: Box::new(Expression::Ident(Ident {
                str: "a",
                span: (3..4).into(),
            })),
            accesser: Box::new(Expression::Primitive(Primitive::String("b"))),
        },
    );
}

#[test]
fn composition_func() {
    let expr = Expression::Invoke {
        expr: Box::new(Expression::Ident(Ident {
            str: "f",
            span: (2..3).into(),
        })),
        args: vec![Expression::Invoke {
            expr: Box::new(Expression::Ident(Ident {
                str: "g",
                span: (4..5).into(),
            })),
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
                expr: Box::new(Expression::Ident(Ident {
                    str: "f",
                    span: (2..3).into(),
                })),
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
                        expr: Box::new(Expression::Ident(Ident {
                            str: "f",
                            span: (3..4).into(),
                        })),
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
            expr: Box::new(Expression::Ident(Ident {
                str: "f",
                span: (8..9).into(),
            })),
            args: vec![],
        },
    );
}

#[test]
fn multiple_dot_access() {
    do_expr_test(
        "a.b.c",
        Expression::DotAccess {
            expr: Box::new(Expression::DotAccess {
                expr: Box::new(Expression::Ident(Ident {
                    str: "a",
                    span: (2..3).into(),
                })),
                accesser: Ident {
                    str: "b",
                    span: (4..5).into(),
                },
            }),
            accesser: Ident {
                str: "c",
                span: (6..7).into(),
            },
        },
    );
}

#[test]
fn multiple_index_access() {
    do_expr_test(
        "a['b'][1]",
        Expression::IndexAccess {
            expr: Box::new(Expression::IndexAccess {
                expr: Box::new(Expression::Ident(Ident {
                    str: "a",
                    span: (2..3).into(),
                })),
                accesser: Box::new(Expression::Primitive(Primitive::String("b"))),
            }),
            accesser: Box::new(Expression::Primitive(Primitive::Int(1))),
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
                    expr: Box::new(Expression::Ident(Ident {
                        str: "a",
                        span: (6..7).into(),
                    })),
                }),
                rhs: Box::new(Expression::Primitive(Primitive::Int(10))),
            }),
            rhs: Box::new(Expression::Binary {
                op: BinaryOp::And,
                lhs: Box::new(Expression::Binary {
                    op: BinaryOp::GreaterEq,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(5))),
                    rhs: Box::new(Expression::Ident(Ident {
                        str: "b",
                        span: (22..23).into(),
                    })),
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
                        lhs: Box::new(Expression::Ident(Ident {
                            str: "a",
                            span: (8..9).into(),
                        })),
                        rhs: Box::new(Expression::Primitive(Primitive::Int(10))),
                    }),
                }),
                rhs: Box::new(Expression::Binary {
                    op: BinaryOp::Less,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(5))),
                    rhs: Box::new(Expression::Ident(Ident {
                        str: "b",
                        span: (24..25).into(),
                    })),
                }),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Bool(false))),
        },
    )
}

#[test]
fn complicated_partt() {
    do_expr_test(
        "-(false or b).c[c.c() and -d()] ** 2",
        Expression::Binary {
            op: BinaryOp::Pow,
            lhs: Box::new(Expression::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(Expression::IndexAccess {
                    expr: Box::new(Expression::DotAccess {
                        expr: Box::new(Expression::Binary {
                            op: BinaryOp::Or,
                            lhs: Box::new(Expression::Primitive(Primitive::Bool(false))),
                            rhs: Box::new(Expression::Ident(Ident {
                                str: "b",
                                span: (13..14).into(),
                            })),
                        }),
                        accesser: Ident {
                            str: "c",
                            span: (16..17).into(),
                        },
                    }),
                    accesser: Box::new(Expression::Binary {
                        op: BinaryOp::And,
                        lhs: Box::new(Expression::Invoke {
                            expr: Box::new(Expression::DotAccess {
                                expr: Box::new(Expression::Ident(Ident {
                                    str: "c",
                                    span: (18..19).into(),
                                })),
                                accesser: Ident {
                                    str: "c",
                                    span: (20..21).into(),
                                },
                            }),
                            args: vec![],
                        }),
                        rhs: Box::new(Expression::Unary {
                            op: UnaryOp::Neg,
                            expr: Box::new(Expression::Invoke {
                                expr: Box::new(Expression::Ident(Ident {
                                    str: "d",
                                    span: (29..30).into(),
                                })),
                                args: vec![],
                            }),
                        }),
                    }),
                }),
            }),
            rhs: Box::new(Expression::Primitive(Primitive::Int(2))),
        },
    );
}
