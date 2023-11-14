mod common;
use parser::tree::*;

fn do_chunk_test(src: &str, chunk: Chunk<'_>) {
    let program = common::parse_program(src);
    let res = program.body;
    assert_eq!(res, chunk);
}

// Variable statement

#[test]
fn define_variable_with_litera() {
    do_chunk_test(
        "var x = 17",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Variable(VariableStatement::Var {
                    name: (Ident("x"), 4..5),
                    expr: (Expression::Primitive(Primitive::Int(17)), 8..10),
                }),
                0..10,
            )],
        },
    )
}

#[test]
fn define_variable_with_func_call() {
    do_chunk_test(
        "var x = f()",
        Chunk {
            captures: vec!["f"],
            body: vec![(
                Statement::Variable(VariableStatement::Var {
                    name: (Ident("x"), 4..5),
                    expr: (
                        Expression::Invoke {
                            expr: (Box::new(Expression::Ident(Ident("f"))), 8..9),
                            args: vec![],
                        },
                        (8..11),
                    ),
                }),
                0..11,
            )],
        },
    )
}

#[test]
fn assign_variable_with_literal() {
    do_chunk_test(
        "x = 1.23",
        Chunk {
            captures: vec!["x"],
            body: vec![(
                Statement::Variable(VariableStatement::Assign {
                    name: (Ident("x"), 0..1),
                    accesser: vec![],
                    expr: (Expression::Primitive(Primitive::Float(1.23)), 4..8),
                }),
                0..8,
            )],
        },
    )
}

#[test]
fn define_function_without_args_and_body() {
    do_chunk_test(
        "func f() end",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Variable(VariableStatement::Func {
                    name: (Ident("f"), 5..6),
                    args: vec![],
                    body: Chunk {
                        captures: vec![],
                        body: vec![],
                    },
                }),
                0..12,
            )],
        },
    );
}

#[test]
fn define_function_with_args_and_body() {
    do_chunk_test(
        "func f(a, b) return 'a' end",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Variable(VariableStatement::Func {
                    name: (Ident("f"), 5..6),
                    args: vec![(Ident("a"), 7..8), (Ident("b"), 10..11)],
                    body: Chunk {
                        captures: vec![],
                        body: vec![(
                            Statement::Control(ControlStatement::Return {
                                value: Some((
                                    (Expression::Primitive(Primitive::String("a".to_string()))),
                                    20..23,
                                )),
                            }),
                            13..23,
                        )],
                    },
                }),
                0..27,
            )],
        },
    );
}

#[test]
fn define_table_field_function() {
    do_chunk_test(
        "func t.a.b() end",
        Chunk {
            captures: vec!["t"],
            body: vec![(
                Statement::Variable(VariableStatement::FieldFunc {
                    table: (Ident("t"), 5..6),
                    fields: vec![(Ident("a"), 7..8), (Ident("b"), 9..10)],
                    args: vec![],
                    body: Chunk {
                        captures: vec![],
                        body: vec![],
                    },
                }),
                0..16,
            )],
        },
    );
}

// Call statement

#[test]
fn call_function_without_args() {
    do_chunk_test(
        "f()",
        Chunk {
            captures: vec!["f"],
            body: vec![(
                Statement::Call(CallStatement::Invoke {
                    expr: (Expression::Ident(Ident("f")), 0..1),
                    args: vec![],
                }),
                0..3,
            )],
        },
    );
}

#[test]
fn call_function_with_args() {
    do_chunk_test(
        "f(1, 'a', true)",
        Chunk {
            captures: vec!["f"],
            body: vec![(
                Statement::Call(CallStatement::Invoke {
                    expr: (Expression::Ident(Ident("f")), 0..1),
                    args: vec![
                        (Expression::Primitive(Primitive::Int(1)), 2..3),
                        (
                            Expression::Primitive(Primitive::String("a".to_string())),
                            5..8,
                        ),
                        (Expression::Primitive(Primitive::Bool(true)), 10..14),
                    ],
                }),
                0..15,
            )],
        },
    );
}

#[test]
fn method_call() {
    do_chunk_test(
        "a->b('a')",
        Chunk {
            captures: vec!["a"],
            body: vec![(
                Statement::Call(CallStatement::MethodCall {
                    expr: (Expression::Ident(Ident("a")), 0..1),
                    name: (Ident("b"), 3..4),
                    args: vec![(
                        Expression::Primitive(Primitive::String("a".to_string())),
                        5..8,
                    )],
                }),
                0..9,
            )],
        },
    );
}

#[test]
fn method_call_obj() {
    do_chunk_test(
        "[1, 2]->len()",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Call(CallStatement::MethodCall {
                    expr: (
                        Expression::ArrayObject(ArrayObject {
                            elements: vec![
                                (Expression::Primitive(Primitive::Int(1)), 1..2),
                                (Expression::Primitive(Primitive::Int(2)), 4..5),
                            ],
                        }),
                        0..6,
                    ),
                    name: (Ident("len"), 8..11),
                    args: vec![],
                }),
                0..13,
            )],
        },
    );
}

// Control statement

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn if_empty() {
    do_chunk_test(
        "if true then end",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Control(ControlStatement::If {
                    cond: (Expression::Primitive(Primitive::Bool(true)), (3..7)),
                    body: Block { body: vec![] },
                    elifs: vec![],
                    else_: None,
                }),
                0..16,
            )],
        },
    );
}

#[test]
fn if_elif_else() {
    do_chunk_test(
        "if true then return 1 elif false then return 2 else return 3 end",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Control(ControlStatement::If {
                    cond: (Expression::Primitive(Primitive::Bool(true)), (3..7)),
                    body: Block {
                        body: vec![(
                            Statement::Control(ControlStatement::Return {
                                value: Some((Expression::Primitive(Primitive::Int(1)), 20..21)),
                            }),
                            13..21,
                        )],
                    },
                    elifs: vec![(
                        (Expression::Primitive(Primitive::Bool(false)), (27..32)),
                        Block {
                            body: vec![(
                                Statement::Control(ControlStatement::Return {
                                    value: Some((Expression::Primitive(Primitive::Int(2)), 45..46)),
                                }),
                                38..46,
                            )],
                        },
                    )],
                    else_: Some(Block {
                        body: vec![(
                            Statement::Control(ControlStatement::Return {
                                value: Some((Expression::Primitive(Primitive::Int(3)), 59..60)),
                            }),
                            52..60,
                        )],
                    }),
                }),
                0..64,
            )],
        },
    )
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn for_in_array() {
    do_chunk_test(
        "for i in [1, 2, 3] do end",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Control(ControlStatement::For {
                    value: (Ident("i"), 4..5),
                    iter: (
                        Expression::ArrayObject(ArrayObject {
                            elements: vec![
                                (Expression::Primitive(Primitive::Int(1)), (10..11)),
                                (Expression::Primitive(Primitive::Int(2)), (13..14)),
                                (Expression::Primitive(Primitive::Int(3)), (16..17)),
                            ],
                        }),
                        9..18,
                    ),
                    body: Block { body: vec![] },
                }),
                0..25,
            )],
        },
    );
}

#[test]
fn for_with_body() {
    do_chunk_test(
        "for i in 1->upto(10) do a = a + i end",
        Chunk {
            captures: vec!["a"],
            body: vec![(
                Statement::Control(ControlStatement::For {
                    value: (Ident("i"), 4..5),
                    iter: (
                        Expression::MethodCall {
                            expr: (Box::new(Expression::Primitive(Primitive::Int(1))), 9..10),
                            name: (Ident("upto"), 12..16),
                            args: vec![(Expression::Primitive(Primitive::Int(10)), 17..19)],
                        },
                        9..20,
                    ),
                    body: Block {
                        body: vec![(
                            Statement::Variable(VariableStatement::Assign {
                                name: (Ident("a"), 24..25),
                                accesser: vec![],
                                expr: (
                                    Expression::Binary {
                                        op: BinaryOp::Add,
                                        lhs: (Box::new(Expression::Ident(Ident("a"))), 28..29),
                                        rhs: (Box::new(Expression::Ident(Ident("i"))), 32..33),
                                    },
                                    28..33,
                                ),
                            }),
                            24..33,
                        )],
                    },
                }),
                0..37,
            )],
        },
    )
}

#[test]
fn while_() {
    do_chunk_test(
        "while ok() do break end",
        Chunk {
            captures: vec!["ok"],
            body: vec![(
                Statement::Control(ControlStatement::While {
                    cond: (
                        Expression::Invoke {
                            expr: (Box::new(Expression::Ident(Ident("ok"))), 6..8),
                            args: vec![],
                        },
                        (6..10),
                    ),
                    body: Block {
                        body: vec![(Statement::Control(ControlStatement::Break), 14..19)],
                    },
                }),
                0..23,
            )],
        },
    );
}

#[test]
fn do_with_no_body() {
    do_chunk_test(
        "do end",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Control(ControlStatement::Do {
                    body: Block { body: vec![] },
                }),
                0..6,
            )],
        },
    );
}

#[test]
fn return_none() {
    do_chunk_test(
        "return",
        Chunk {
            captures: vec![],
            body: vec![(
                Statement::Control(ControlStatement::Return { value: None }),
                0..6,
            )],
        },
    );
}
