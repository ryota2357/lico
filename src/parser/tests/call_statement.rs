mod common;
use parser::tree::*;

fn do_statement_test(src: &str, statement: Statement<'_>) {
    let program = common::parse_program(src);
    let stats = program.body.block;
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].1, (0..src.len()));
    assert_eq!(stats[0].0, statement);
}

#[test]
fn complicated_func_with_trailing_comma() {
    do_statement_test(
        "f(g(),)",
        Statement::Call(CallStatement::Invoke {
            expr: (Expression::Ident(Ident("f")), 0..1),
            args: vec![(
                Expression::Invoke {
                    expr: (Box::new(Expression::Ident(Ident("g"))), 2..3),
                    args: vec![],
                },
                2..5,
            )],
        }),
    );
}

#[test]
#[should_panic] // TODO: 構文エラーレポートのテストを書く
fn call_with_only_comma() {
    do_statement_test(
        "f(,)",
        Statement::Call(CallStatement::Invoke {
            expr: (Expression::Ident(Ident("f")), 0..1),
            args: vec![],
        }),
    );
}

#[test]
fn multiple_call() {
    do_statement_test(
        "f()()",
        Statement::Call(CallStatement::Invoke {
            expr: (
                Expression::Invoke {
                    expr: (Box::new(Expression::Ident(Ident("f"))), 0..1),
                    args: vec![],
                },
                0..3,
            ),
            args: vec![],
        }),
    );
}

#[test]
fn multiple_call_more() {
    do_statement_test(
        "f(1)(2)(3)",
        Statement::Call(CallStatement::Invoke {
            expr: (
                Expression::Invoke {
                    expr: (
                        Box::new(Expression::Invoke {
                            expr: (Box::new(Expression::Ident(Ident("f"))), 0..1),
                            args: vec![(Expression::Primitive(Primitive::Int(1)), 2..3)],
                        }),
                        0..4,
                    ),
                    args: vec![(Expression::Primitive(Primitive::Int(2)), 5..6)],
                },
                0..7,
            ),
            args: vec![(Expression::Primitive(Primitive::Int(3)), 8..9)],
        }),
    );
}

#[test]
fn delimited_call() {
    do_statement_test(
        "((f(1))(2))(3)",
        Statement::Call(CallStatement::Invoke {
            expr: (
                Expression::Invoke {
                    expr: (
                        Box::new(Expression::Invoke {
                            expr: (Box::new(Expression::Ident(Ident("f"))), 2..3),
                            args: vec![(Expression::Primitive(Primitive::Int(1)), 4..5)],
                        }),
                        2..6,
                    ),
                    args: vec![(Expression::Primitive(Primitive::Int(2)), 8..9)],
                },
                1..10,
            ),
            args: vec![(Expression::Primitive(Primitive::Int(3)), 12..13)],
        }),
    );
}

#[test]
fn multiple_call_with_delimited() {
    do_statement_test(
        "(f(1)(2))(3)(4)",
        Statement::Call(CallStatement::Invoke {
            expr: (
                Expression::Invoke {
                    expr: (
                        Box::new(Expression::Invoke {
                            expr: (
                                Box::new(Expression::Invoke {
                                    expr: (Box::new(Expression::Ident(Ident("f"))), 1..2),
                                    args: vec![(Expression::Primitive(Primitive::Int(1)), 3..4)],
                                }),
                                1..5,
                            ),
                            args: vec![(Expression::Primitive(Primitive::Int(2)), 6..7)],
                        }),
                        1..8,
                    ),
                    args: vec![(Expression::Primitive(Primitive::Int(3)), 10..11)],
                },
                1..12,
            ),
            args: vec![(Expression::Primitive(Primitive::Int(4)), 13..14)],
        }),
    );
}

#[test]
fn method_chain() {
    do_statement_test(
        "a->b()->c()",
        Statement::Call(CallStatement::MethodCall {
            expr: (
                Expression::MethodCall {
                    expr: (Box::new(Expression::Ident(Ident("a"))), 0..1),
                    name: (Ident("b"), (3..4)),
                    args: vec![],
                },
                0..6,
            ),
            name: (Ident("c"), (8..9)),
            args: vec![],
        }),
    );
    do_statement_test(
        "(a->b(1))->c(2, 3)",
        Statement::Call(CallStatement::MethodCall {
            expr: (
                Expression::MethodCall {
                    expr: (Box::new(Expression::Ident(Ident("a"))), 1..2),
                    name: (Ident("b"), (4..5)),
                    args: vec![(Expression::Primitive(Primitive::Int(1)), 6..7)],
                },
                1..8,
            ),
            name: (Ident("c"), (11..12)),
            args: vec![
                (Expression::Primitive(Primitive::Int(2)), 13..14),
                (Expression::Primitive(Primitive::Int(3)), 16..17),
            ],
        }),
    );
}

#[test]
fn multiple_call_with_method() {
    do_statement_test(
        "a->b(1)(false)->c('3')",
        Statement::Call(CallStatement::MethodCall {
            expr: (
                Expression::Invoke {
                    expr: (
                        Box::new(Expression::MethodCall {
                            expr: (Box::new(Expression::Ident(Ident("a"))), 0..1),
                            name: (Ident("b"), (3..4)),
                            args: vec![(Expression::Primitive(Primitive::Int(1)), 5..6)],
                        }),
                        0..7,
                    ),
                    args: vec![(Expression::Primitive(Primitive::Bool(false)), 8..13)],
                },
                0..14,
            ),
            name: (Ident("c"), (16..17)),
            args: vec![(
                Expression::Primitive(Primitive::String("3".to_string())),
                18..21,
            )],
        }),
    );
}

#[test]
fn anonymous_func_call() {
    do_statement_test(
        "(func(x) return x end)(1)",
        Statement::Call(CallStatement::Invoke {
            expr: (
                Expression::FunctionObject(FunctionObject {
                    args: vec![(Ident("x"), 6..7)],
                    body: Chunk {
                        captures: vec![],
                        block: vec![(
                            Statement::Control(ControlStatement::Return {
                                value: Some((Expression::Ident(Ident("x")), 16..17)),
                            }),
                            9..17,
                        )],
                    },
                }),
                1..21,
            ),
            args: vec![(Expression::Primitive(Primitive::Int(1)), 23..24)],
        }),
    );
}
