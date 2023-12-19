mod common;
use parser::tree::*;

fn do_expr_test(src: &str, expression: Expression<'_>) {
    let src = format!("_={}", src);
    let program = common::parse_program(&src);
    let stats = program.body.block;
    assert_eq!(stats.len(), 1);
    let statement = &stats[0];
    if let Statement::Variable(VariableStatement::Assign { expr, .. }) = &statement.0 {
        assert_eq!(expr.0, expression);
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
                block: Block(vec![]),
            },
        }),
    );
    do_expr_test(
        "func(a, b) return c end",
        Expression::FunctionObject(FunctionObject {
            args: vec![Ident("a", 7..8), Ident("b", 10..11)],
            body: Chunk {
                captures: vec![("c", 20..21)],
                block: Block(vec![(
                    Statement::Control(ControlStatement::Return {
                        value: Some((Expression::Ident(Ident("c", 20..21)), 20..21)),
                    }),
                    13..21,
                )]),
            },
        }),
    )
}

#[test]
fn array_object() {
    do_expr_test("[]", Expression::ArrayObject(ArrayObject(vec![])));
    do_expr_test(
        "[1, [true, 'a'], {}]",
        Expression::ArrayObject(ArrayObject(vec![
            (Expression::Primitive(Primitive::Int(1)), 3..4),
            (
                Expression::ArrayObject(ArrayObject(vec![
                    (Expression::Primitive(Primitive::Bool(true)), 7..11),
                    (
                        Expression::Primitive(Primitive::String("a".to_string())),
                        13..16,
                    ),
                ])),
                6..17,
            ),
            (Expression::TableObject(TableObject(vec![])), 19..21),
        ])),
    );
}

#[test]
fn table_object() {
    do_expr_test("{}", Expression::TableObject(TableObject(vec![])));
    do_expr_test(
        "{ a = 1, b = {a=1}, }",
        Expression::TableObject(TableObject(vec![
            (
                (
                    Expression::Primitive(Primitive::String("a".to_string())),
                    4..5,
                ),
                (Expression::Primitive(Primitive::Int(1)), 8..9),
            ),
            (
                (
                    Expression::Primitive(Primitive::String("b".to_string())),
                    11..12,
                ),
                (
                    Expression::TableObject(TableObject(vec![(
                        (
                            Expression::Primitive(Primitive::String("a".to_string())),
                            16..17,
                        ),
                        (Expression::Primitive(Primitive::Int(1)), 18..19),
                    )])),
                    15..20,
                ),
            ),
        ])),
    );
}

#[test]
fn delimited_call() {
    do_expr_test(
        "(f())",
        Expression::Invoke {
            expr: (Box::new(Expression::Ident(Ident("f", 3..4))), 3..4),
            args: vec![],
        },
    );
    do_expr_test(
        "(((f(1))))",
        Expression::Invoke {
            expr: (Box::new(Expression::Ident(Ident("f", 5..6))), 5..6),
            args: vec![(Expression::Primitive(Primitive::Int(1)), 7..8)],
        },
    );
}

#[test]
fn delimited_pratt() {
    do_expr_test(
        "(1+2)",
        Expression::Binary {
            op: BinaryOp::Add,
            lhs: (Box::new(Expression::Primitive(Primitive::Int(1))), 3..4),
            rhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 5..6),
        },
    );
    do_expr_test(
        "(((1+2*3)))",
        Expression::Binary {
            op: BinaryOp::Add,
            lhs: (Box::new(Expression::Primitive(Primitive::Int(1))), 5..6),
            rhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Mul,
                    lhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 7..8),
                    rhs: (Box::new(Expression::Primitive(Primitive::Int(3))), 9..10),
                }),
                7..10,
            ),
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
    do_expr_test("(a)", Expression::Ident(Ident("a", 3..4)));
    do_expr_test("(((a)))", Expression::Ident(Ident("a", 5..6)));
}
#[test]
fn delimited_local_access() {
    do_expr_test(
        "((a.b).c)",
        Expression::DotAccess {
            expr: (
                Box::new(Expression::DotAccess {
                    expr: (Box::new(Expression::Ident(Ident("a", 4..5))), 4..5),
                    accesser: Ident("b", 6..7),
                }),
                (4..7),
            ),
            accesser: Ident("c", 9..10),
        },
    );
    do_expr_test(
        "(a['b'])",
        Expression::IndexAccess {
            expr: (Box::new(Expression::Ident(Ident("a", 3..4))), 3..4),
            accesser: (
                Box::new(Expression::Primitive(Primitive::String("b".to_string()))),
                5..8,
            ),
        },
    );
}

#[test]
fn composition_func() {
    let expr = Expression::Invoke {
        expr: (Box::new(Expression::Ident(Ident("f", 2..3))), 2..3),
        args: vec![(
            Expression::Invoke {
                expr: (Box::new(Expression::Ident(Ident("g", 4..5))), 4..5),
                args: vec![],
            },
            4..7,
        )],
    };
    do_expr_test("f(g())", expr.clone());
    do_expr_test("f(g(),)", expr);
}

#[test]
fn multiple_call() {
    do_expr_test(
        "f()()",
        Expression::Invoke {
            expr: (
                Box::new(Expression::Invoke {
                    expr: (Box::new(Expression::Ident(Ident("f", 2..3))), 2..3),
                    args: vec![],
                }),
                2..5,
            ),
            args: vec![],
        },
    );
    do_expr_test(
        "(f(1)(2))()(false)",
        Expression::Invoke {
            expr: (
                Box::new(Expression::Invoke {
                    expr: (
                        Box::new(Expression::Invoke {
                            expr: (
                                Box::new(Expression::Invoke {
                                    expr: (Box::new(Expression::Ident(Ident("f", 3..4))), 3..4),
                                    args: vec![(Expression::Primitive(Primitive::Int(1)), 5..6)],
                                }),
                                3..7,
                            ),
                            args: vec![(Expression::Primitive(Primitive::Int(2)), 8..9)],
                        }),
                        3..10,
                    ),
                    args: vec![],
                }),
                2..13,
            ),
            args: vec![(Expression::Primitive(Primitive::Bool(false)), 14..19)],
        },
    );
}

#[test]
fn method_chain() {
    do_expr_test(
        "a->b()->c()",
        Expression::MethodCall {
            expr: (
                Box::new(Expression::MethodCall {
                    expr: (Box::new(Expression::Ident(Ident("a", 2..3))), 2..3),
                    name: Ident("b", 5..6),
                    args: vec![],
                }),
                2..8,
            ),
            name: Ident("c", 10..11),
            args: vec![],
        },
    );
    do_expr_test(
        "((a->b(1))->c(2, 3))",
        Expression::MethodCall {
            expr: (
                Box::new(Expression::MethodCall {
                    expr: (Box::new(Expression::Ident(Ident("a", 4..5))), 4..5),
                    name: Ident("b", 7..8),
                    args: vec![(Expression::Primitive(Primitive::Int(1)), 9..10)],
                }),
                4..11,
            ),
            name: Ident("c", 14..15),
            args: vec![
                (Expression::Primitive(Primitive::Int(2)), 16..17),
                (Expression::Primitive(Primitive::Int(3)), 19..20),
            ],
        },
    )
}

#[test]
fn multiple_call_with_method() {
    do_expr_test(
        "a->b(1)(2)->c(3)",
        Expression::MethodCall {
            expr: (
                Box::new(Expression::Invoke {
                    expr: (
                        Box::new(Expression::MethodCall {
                            expr: (Box::new(Expression::Ident(Ident("a", 2..3))), 2..3),
                            name: Ident("b", 5..6),
                            args: vec![(Expression::Primitive(Primitive::Int(1)), 7..8)],
                        }),
                        2..9,
                    ),
                    args: vec![(Expression::Primitive(Primitive::Int(2)), 10..11)],
                }),
                2..12,
            ),
            name: Ident("c", 14..15),
            args: vec![(Expression::Primitive(Primitive::Int(3)), 16..17)],
        },
    );
}

#[test]
#[should_panic]
fn error_func_call() {
    do_expr_test(
        "f(,)",
        Expression::Invoke {
            expr: (Box::new(Expression::Ident(Ident("f", 2..3))), 2..3),
            args: vec![],
        },
    );
}

#[test]
fn multiple_dot_access() {
    do_expr_test(
        "a.b.c",
        Expression::DotAccess {
            expr: (
                Box::new(Expression::DotAccess {
                    expr: (Box::new(Expression::Ident(Ident("a", 2..3))), 2..3),
                    accesser: Ident("b", 4..5),
                }),
                2..5,
            ),
            accesser: Ident("c", 6..7),
        },
    );
}

#[test]
fn multiple_index_access() {
    do_expr_test(
        "a['b'][1]",
        Expression::IndexAccess {
            expr: (
                Box::new(Expression::IndexAccess {
                    expr: (Box::new(Expression::Ident(Ident("a", 2..3))), 2..3),
                    accesser: (
                        Box::new(Expression::Primitive(Primitive::String("b".to_string()))),
                        4..7,
                    ),
                }),
                2..8,
            ),
            accesser: (Box::new(Expression::Primitive(Primitive::Int(1))), 9..10),
        },
    );
}

#[test]
fn arithmetic_op() {
    do_expr_test(
        "-1 + 2 * 3 / 4 - 5", // ((-1) + ((2 * 3) / 4)) - 5
        Expression::Binary {
            op: BinaryOp::Sub,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Add,
                    lhs: (Box::new(Expression::Primitive(Primitive::Int(-1))), 2..4),
                    rhs: (
                        Box::new(Expression::Binary {
                            op: BinaryOp::Div,
                            lhs: (
                                Box::new(Expression::Binary {
                                    op: BinaryOp::Mul,
                                    lhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 7..8),
                                    rhs: (
                                        Box::new(Expression::Primitive(Primitive::Int(3))),
                                        11..12,
                                    ),
                                }),
                                7..12,
                            ),
                            rhs: (Box::new(Expression::Primitive(Primitive::Int(4))), 15..16),
                        }),
                        7..16,
                    ),
                }),
                2..16,
            ),
            rhs: (Box::new(Expression::Primitive(Primitive::Int(5))), 19..20),
        },
    );
    do_expr_test(
        "1 * 2 ** 3 ** 4 / 5", // (1 * (2 ** (3 ** 4))) / 5
        Expression::Binary {
            op: BinaryOp::Div,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Mul,
                    lhs: (Box::new(Expression::Primitive(Primitive::Int(1))), 2..3),
                    rhs: (
                        Box::new(Expression::Binary {
                            op: BinaryOp::Pow,
                            lhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 6..7),
                            rhs: (
                                Box::new(Expression::Binary {
                                    op: BinaryOp::Pow,
                                    lhs: (
                                        Box::new(Expression::Primitive(Primitive::Int(3))),
                                        11..12,
                                    ),
                                    rhs: (
                                        Box::new(Expression::Primitive(Primitive::Int(4))),
                                        16..17,
                                    ),
                                }),
                                11..17,
                            ),
                        }),
                        6..17,
                    ),
                }),
                2..17,
            ),
            rhs: (Box::new(Expression::Primitive(Primitive::Int(5))), 20..21),
        },
    );
}

#[test]
fn arithmetic_op_with_paren() {
    do_expr_test(
        "(-1 + 2) * 3",
        Expression::Binary {
            op: BinaryOp::Mul,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Add,
                    lhs: (Box::new(Expression::Primitive(Primitive::Int(-1))), 3..5),
                    rhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 8..9),
                }),
                3..9,
            ),
            rhs: (Box::new(Expression::Primitive(Primitive::Int(3))), 13..14),
        },
    );
    do_expr_test(
        "(1 + (2 ** 3)) ** 4",
        Expression::Binary {
            op: BinaryOp::Pow,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Add,
                    lhs: (Box::new(Expression::Primitive(Primitive::Int(1))), 3..4),
                    rhs: (
                        Box::new(Expression::Binary {
                            op: BinaryOp::Pow,
                            lhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 8..9),
                            rhs: (Box::new(Expression::Primitive(Primitive::Int(3))), 13..14),
                        }),
                        8..14,
                    ),
                }),
                3..14,
            ),
            rhs: (Box::new(Expression::Primitive(Primitive::Int(4))), 20..21),
        },
    )
}

#[test]
fn logical_op() {
    do_expr_test(
        "not a == 10 or 5 >= b and false", // ((not a) == 10) or ((5 >= b) and false)
        Expression::Binary {
            op: BinaryOp::Or,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Eq,
                    lhs: (
                        Box::new(Expression::Unary {
                            op: UnaryOp::Not,
                            expr: (Box::new(Expression::Ident(Ident("a", 6..7))), 6..7),
                        }),
                        2..7,
                    ),
                    rhs: (Box::new(Expression::Primitive(Primitive::Int(10))), 11..13),
                }),
                2..13,
            ),
            rhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::And,
                    lhs: (
                        Box::new(Expression::Binary {
                            op: BinaryOp::GreaterEq,
                            lhs: (Box::new(Expression::Primitive(Primitive::Int(5))), 17..18),
                            rhs: (Box::new(Expression::Ident(Ident("b", 22..23))), 22..23),
                        }),
                        17..23,
                    ),
                    rhs: (
                        Box::new(Expression::Primitive(Primitive::Bool(false))),
                        28..33,
                    ),
                }),
                17..33,
            ),
        },
    );
}

#[test]
fn logical_op_with_paren() {
    do_expr_test(
        "(not (a != 10) or 5 < b) and false",
        Expression::Binary {
            op: BinaryOp::And,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Or,
                    lhs: (
                        Box::new(Expression::Unary {
                            op: UnaryOp::Not,
                            expr: (
                                Box::new(Expression::Binary {
                                    op: BinaryOp::NotEq,
                                    lhs: (Box::new(Expression::Ident(Ident("a", 8..9))), 8..9),
                                    rhs: (
                                        Box::new(Expression::Primitive(Primitive::Int(10))),
                                        13..15,
                                    ),
                                }),
                                8..15,
                            ),
                        }),
                        3..16,
                    ),
                    rhs: (
                        Box::new(Expression::Binary {
                            op: BinaryOp::Less,
                            lhs: (Box::new(Expression::Primitive(Primitive::Int(5))), 20..21),
                            rhs: (Box::new(Expression::Ident(Ident("b", 24..25))), 24..25),
                        }),
                        20..25,
                    ),
                }),
                3..25,
            ),
            rhs: (
                Box::new(Expression::Primitive(Primitive::Bool(false))),
                31..36,
            ),
        },
    )
}

#[test]
fn str_concat() {
    do_expr_test(
        "'a' .. 4 + 1 == 'a5'",
        Expression::Binary {
            op: BinaryOp::Eq,
            lhs: (
                Box::new(Expression::Binary {
                    op: BinaryOp::Concat,
                    lhs: (
                        Box::new(Expression::Primitive(Primitive::String("a".to_string()))),
                        2..5,
                    ),
                    rhs: (
                        Box::new(Expression::Binary {
                            op: BinaryOp::Add,
                            lhs: (Box::new(Expression::Primitive(Primitive::Int(4))), 9..10),
                            rhs: (Box::new(Expression::Primitive(Primitive::Int(1))), 13..14),
                        }),
                        9..14,
                    ),
                }),
                2..14,
            ),
            rhs: (
                Box::new(Expression::Primitive(Primitive::String("a5".to_string()))),
                18..22,
            ),
        },
    );
}

#[test]
fn complicated_pratt() {
    do_expr_test(
        "-(false or b).c[c.c() and -d()] ** 2",
        Expression::Binary {
            op: BinaryOp::Pow,
            lhs: (
                Box::new(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: (
                        Box::new(Expression::IndexAccess {
                            expr: (
                                Box::new(Expression::DotAccess {
                                    expr: (
                                        Box::new(Expression::Binary {
                                            op: BinaryOp::Or,
                                            lhs: (
                                                Box::new(Expression::Primitive(Primitive::Bool(
                                                    false,
                                                ))),
                                                4..9,
                                            ),
                                            rhs: (
                                                Box::new(Expression::Ident(Ident("b", 13..14))),
                                                13..14,
                                            ),
                                        }),
                                        4..14,
                                    ),
                                    accesser: Ident("c", 16..17),
                                }),
                                3..17,
                            ),
                            accesser: (
                                Box::new(Expression::Binary {
                                    op: BinaryOp::And,
                                    lhs: (
                                        Box::new(Expression::Invoke {
                                            expr: (
                                                Box::new(Expression::DotAccess {
                                                    expr: (
                                                        Box::new(Expression::Ident(Ident(
                                                            "c",
                                                            18..19,
                                                        ))),
                                                        18..19,
                                                    ),
                                                    accesser: Ident("c", 20..21),
                                                }),
                                                18..21,
                                            ),
                                            args: vec![],
                                        }),
                                        18..23,
                                    ),
                                    rhs: (
                                        Box::new(Expression::Unary {
                                            op: UnaryOp::Neg,
                                            expr: (
                                                Box::new(Expression::Invoke {
                                                    expr: (
                                                        Box::new(Expression::Ident(Ident(
                                                            "d",
                                                            29..30,
                                                        ))),
                                                        29..30,
                                                    ),
                                                    args: vec![],
                                                }),
                                                29..32,
                                            ),
                                        }),
                                        28..32,
                                    ),
                                }),
                                18..32,
                            ),
                        }),
                        3..33,
                    ),
                }),
                2..33,
            ),
            rhs: (Box::new(Expression::Primitive(Primitive::Int(2))), 37..38),
        },
    );
}
