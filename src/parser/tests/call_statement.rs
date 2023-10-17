mod common;
use parser::tree::*;

fn do_statement_test(src: &str, statement: Statement<'_>) {
    let program = common::parse_program(src);
    let stats = program.body.body;
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0], statement);
}

#[test]
fn complicated_func_with_trailing_comma() {
    do_statement_test(
        "f(g(),)",
        Statement::Call(CallStatement::Invoke {
            expr: Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (0..1).into(),
            })),
            args: vec![Expression::Invoke {
                expr: Box::new(Expression::Local(Local::Ident(Ident {
                    str: "g",
                    span: (2..3).into(),
                }))),
                args: vec![],
            }],
        }),
    );
}

#[test]
#[should_panic] // TODO: 構文エラーレポートのテストを書く
fn call_with_only_comma() {
    // do_statement_test(
    //     "f(,)",
    //     None
    // );
    do_statement_test(
        "f(,)",
        Statement::Call(CallStatement::Invoke {
            expr: Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (0..1).into(),
            })),
            args: vec![],
        }),
    );
}

#[test]
fn multiple_call() {
    do_statement_test(
        "f()()",
        Statement::Call(CallStatement::Invoke {
            expr: Expression::Invoke {
                expr: Box::new(Expression::Local(Local::Ident(Ident {
                    str: "f",
                    span: (0..1).into(),
                }))),
                args: vec![],
            },
            args: vec![],
        }),
    );
}

#[test]
fn multiple_call_more() {
    do_statement_test(
        "f(1)(2)(3)",
        Statement::Call(CallStatement::Invoke {
            expr: Expression::Invoke {
                expr: Box::new(Expression::Invoke {
                    expr: Box::new(Expression::Local(Local::Ident(Ident {
                        str: "f",
                        span: (0..1).into(),
                    }))),
                    args: vec![Expression::Primitive(Primitive::Int(1))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(2))],
            },
            args: vec![Expression::Primitive(Primitive::Int(3))],
        }),
    );
}

#[test]
fn delimited_call() {
    do_statement_test(
        "((f(1))(2))(3)",
        Statement::Call(CallStatement::Invoke {
            expr: Expression::Invoke {
                expr: Box::new(Expression::Invoke {
                    expr: Box::new(Expression::Local(Local::Ident(Ident {
                        str: "f",
                        span: (2..3).into(),
                    }))),
                    args: vec![Expression::Primitive(Primitive::Int(1))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(2))],
            },
            args: vec![Expression::Primitive(Primitive::Int(3))],
        }),
    );
}

#[test]
fn multiple_call_with_delimited() {
    do_statement_test(
        "(f(1)(2))(3)(4)",
        Statement::Call(CallStatement::Invoke {
            expr: Expression::Invoke {
                expr: Box::new(Expression::Invoke {
                    expr: Box::new(Expression::Invoke {
                        expr: Box::new(Expression::Local(Local::Ident(Ident {
                            str: "f",
                            span: (1..2).into(),
                        }))),
                        args: vec![Expression::Primitive(Primitive::Int(1))],
                    }),
                    args: vec![Expression::Primitive(Primitive::Int(2))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(3))],
            },
            args: vec![Expression::Primitive(Primitive::Int(4))],
        }),
    );
}
