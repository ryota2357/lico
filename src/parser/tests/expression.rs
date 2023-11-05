mod common;
use parser::tree::*;

fn do_expr_test(src: &str, expression: Expression<'_>) {
    let src = format!("_={}", src);
    let program = common::parse_program(&src);
    let stats = program.body.body;
    assert_eq!(stats.len(), 1);
    let statement = &stats[0];
    if let Statement::Variable(VariableStatement::Assign { expr, .. }) = statement {
        assert_eq!(expr, &expression);
    } else {
        panic!(
            "Expected VariableStatement::Assign, but got {:?}",
            statement
        );
    }
}

#[test]
fn function_object() {
    do_expr_test(
        "func() end",
        Expression::FunctionObject(FunctionObject {
            args: vec![],
            body: Chunk {
                captures: vec![],
                body: vec![],
            },
        }),
    );
    do_expr_test(
        "func(a, b) return c end",
        Expression::FunctionObject(FunctionObject {
            args: vec![
                Ident {
                    str: "a",
                    span: (7..8).into(),
                },
                Ident {
                    str: "b",
                    span: (10..11).into(),
                },
            ],
            body: Chunk {
                captures: vec!["c"],
                body: vec![Statement::Control(ControlStatement::Return {
                    value: Some(Expression::Ident(Ident {
                        str: "c",
                        span: (20..21).into(),
                    })),
                })],
            },
        }),
    )
}

#[test]
fn array_object() {
    do_expr_test(
        "[]",
        Expression::ArrayObject(ArrayObject { elements: vec![] }),
    );
    do_expr_test(
        "[1, [true, 'a'], {}]",
        Expression::ArrayObject(ArrayObject {
            elements: vec![
                Expression::Primitive(Primitive::Int(1)),
                Expression::ArrayObject(ArrayObject {
                    elements: vec![
                        Expression::Primitive(Primitive::Bool(true)),
                        Expression::Primitive(Primitive::String("a".to_string())),
                    ],
                }),
                Expression::TableObject(TableObject { key_values: vec![] }),
            ],
        }),
    )
}

#[test]
fn table_object() {
    do_expr_test(
        "{}",
        Expression::TableObject(TableObject { key_values: vec![] }),
    );
    do_expr_test(
        "{ a = 1, b = {a=1}, }",
        Expression::TableObject(TableObject {
            key_values: vec![
                (
                    Expression::Ident(Ident {
                        str: "a",
                        span: (4..5).into(),
                    }),
                    Expression::Primitive(Primitive::Int(1)),
                ),
                (
                    Expression::Ident(Ident {
                        str: "b",
                        span: (11..12).into(),
                    }),
                    Expression::TableObject(TableObject {
                        key_values: vec![(
                            Expression::Ident(Ident {
                                str: "a",
                                span: (16..17).into(),
                            }),
                            Expression::Primitive(Primitive::Int(1)),
                        )],
                    }),
                ),
            ],
        }),
    );
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
        Expression::Primitive(Primitive::String("abc".to_string())),
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
            accesser: Box::new(Expression::Primitive(Primitive::String("b".to_string()))),
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
fn method_chain() {
    do_expr_test(
        "a->b()->c()",
        Expression::MethodCall {
            expr: Box::new(Expression::MethodCall {
                expr: Box::new(Expression::Ident(Ident {
                    str: "a",
                    span: (2..3).into(),
                })),
                name: Ident {
                    str: "b",
                    span: (5..6).into(),
                },
                args: vec![],
            }),
            name: Ident {
                str: "c",
                span: (10..11).into(),
            },
            args: vec![],
        },
    );
    do_expr_test(
        "((a->b(1))->c(2, 3))",
        Expression::MethodCall {
            expr: Box::new(Expression::MethodCall {
                expr: Box::new(Expression::Ident(Ident {
                    str: "a",
                    span: (4..5).into(),
                })),
                name: Ident {
                    str: "b",
                    span: (7..8).into(),
                },
                args: vec![Expression::Primitive(Primitive::Int(1))],
            }),
            name: Ident {
                str: "c",
                span: (14..15).into(),
            },
            args: vec![
                Expression::Primitive(Primitive::Int(2)),
                Expression::Primitive(Primitive::Int(3)),
            ],
        },
    )
}

#[test]
fn multiple_call_with_method() {
    do_expr_test(
        "a->b(1)(2)->c(3)",
        Expression::MethodCall {
            expr: Box::new(Expression::Invoke {
                expr: Box::new(Expression::MethodCall {
                    expr: Box::new(Expression::Ident(Ident {
                        str: "a",
                        span: (2..3).into(),
                    })),
                    name: Ident {
                        str: "b",
                        span: (5..6).into(),
                    },
                    args: vec![Expression::Primitive(Primitive::Int(1))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(2))],
            }),
            name: Ident {
                str: "c",
                span: (14..15).into(),
            },
            args: vec![Expression::Primitive(Primitive::Int(3))],
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
                accesser: Box::new(Expression::Primitive(Primitive::String("b".to_string()))),
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
