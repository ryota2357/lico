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
        Statement::Call(Call::Local {
            local: Local::Variable {
                name: Ident {
                    str: "f",
                    span: (0..1).into(),
                },
            },
            args: vec![Expression::Call(Call::Local {
                local: Local::Variable {
                    name: Ident {
                        str: "g",
                        span: (2..3).into(),
                    },
                },
                args: vec![],
            })],
        }),
    );
}

#[test]
fn complicated_func_with_trailing_comma_as_expression() {
    do_statement_test(
        "var _ = f(g(),)",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Call(Call::Local {
                local: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (8..9).into(),
                    },
                },
                args: vec![Expression::Call(Call::Local {
                    local: Local::Variable {
                        name: Ident {
                            str: "g",
                            span: (10..11).into(),
                        },
                    },
                    args: vec![],
                })],
            }),
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
        Statement::Call(Call::Local {
            local: Local::Variable {
                name: Ident {
                    str: "f",
                    span: (0..1).into(),
                },
            },
            args: vec![],
        }),
    );
}

#[test]
#[should_panic] // TODO: 構文エラーレポートのテストを書く
fn call_with_only_comma_as_expression() {
    // do_statement_test(
    //     "var _ = f(,)",
    //     None
    // );
    do_statement_test(
        "var _ = f(,)",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Call(Call::Local {
                local: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (8..9).into(),
                    },
                },
                args: vec![],
            }),
        }),
    );
}

#[test]
fn multiple_call() {
    do_statement_test(
        "f()()",
        Statement::Call(Call::Nested {
            call: Box::new(Call::Local {
                local: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (0..1).into(),
                    },
                },
                args: vec![],
            }),
            args: vec![],
        }),
    );
}

#[test]
fn multiple_call_as_expression() {
    do_statement_test(
        "var _ = f()()",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Call(Call::Nested {
                call: Box::new(Call::Local {
                    local: Local::Variable {
                        name: Ident {
                            str: "f",
                            span: (8..9).into(),
                        },
                    },
                    args: vec![],
                }),
                args: vec![],
            }),
        }),
    );
}

#[test]
fn multiple_call_more() {
    do_statement_test(
        "f(1)(2)(3)",
        Statement::Call(Call::Nested {
            call: Box::new(Call::Nested {
                call: Box::new(Call::Local {
                    local: Local::Variable {
                        name: Ident {
                            str: "f",
                            span: (0..1).into(),
                        },
                    },
                    args: vec![Expression::Primitive(Primitive::Int(1))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(2))],
            }),
            args: vec![Expression::Primitive(Primitive::Int(3))],
        }),
    );
}

#[test]
fn multiple_call_more_as_expression() {
    do_statement_test(
        "var _ = f(1)(2)(3)",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Call(Call::Nested {
                call: Box::new(Call::Nested {
                    call: Box::new(Call::Local {
                        local: Local::Variable {
                            name: Ident {
                                str: "f",
                                span: (8..9).into(),
                            },
                        },
                        args: vec![Expression::Primitive(Primitive::Int(1))],
                    }),
                    args: vec![Expression::Primitive(Primitive::Int(2))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(3))],
            }),
        }),
    );
}

#[test]
fn delimited_call() {
    do_statement_test(
        "((f(1))(2))(3)",
        Statement::Call(Call::Nested {
            call: Box::new(Call::Nested {
                call: Box::new(Call::Local {
                    local: Local::Variable {
                        name: Ident {
                            str: "f",
                            span: (2..3).into(),
                        },
                    },
                    args: vec![Expression::Primitive(Primitive::Int(1))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(2))],
            }),
            args: vec![Expression::Primitive(Primitive::Int(3))],
        }),
    );
}

#[test]
fn delimited_call_as_expression() {
    do_statement_test(
        "var _ = ((f(1))(2))(3)",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Call(Call::Nested {
                call: Box::new(Call::Nested {
                    call: Box::new(Call::Local {
                        local: Local::Variable {
                            name: Ident {
                                str: "f",
                                span: (10..11).into(),
                            },
                        },
                        args: vec![Expression::Primitive(Primitive::Int(1))],
                    }),
                    args: vec![Expression::Primitive(Primitive::Int(2))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(3))],
            }),
        }),
    );
}

#[test]
fn multiple_call_with_delimited() {
    do_statement_test(
        "(f(1)(2))(3)(4)",
        Statement::Call(Call::Nested {
            call: Box::new(Call::Nested {
                call: Box::new(Call::Nested {
                    call: Box::new(Call::Local {
                        local: Local::Variable {
                            name: Ident {
                                str: "f",
                                span: (1..2).into(),
                            },
                        },
                        args: vec![Expression::Primitive(Primitive::Int(1))],
                    }),
                    args: vec![Expression::Primitive(Primitive::Int(2))],
                }),
                args: vec![Expression::Primitive(Primitive::Int(3))],
            }),
            args: vec![Expression::Primitive(Primitive::Int(4))],
        }),
    );
}

#[test]
fn complicated_call_with_var() {
    do_statement_test(
        "var _ = (f(a)())()(false)",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Call(Call::Nested {
                call: Box::new(Call::Nested {
                    call: Box::new(Call::Nested {
                        call: Box::new(Call::Local {
                            local: Local::Variable {
                                name: Ident {
                                    str: "f",
                                    span: (9..10).into(),
                                },
                            },
                            args: vec![Expression::Local(Local::Variable {
                                name: Ident {
                                    str: "a",
                                    span: (11..12).into(),
                                },
                            })],
                        }),
                        args: vec![],
                    }),
                    args: vec![],
                }),
                args: vec![Expression::Primitive(Primitive::Bool(false))],
            }),
        }),
    );
}
