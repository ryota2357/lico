mod common;
use parser::tree::*;

// Variable statement

#[test]
fn define_variable_with_literal_by_va() {
    common::do_chunk_test(
        "var x = 17",
        Chunk {
            capture: vec![],
            body: vec![Statement::Variable(VariableStatement::Var {
                name: Ident {
                    str: "x",
                    span: (4..5).into(),
                },
                expr: Expression::Primitive(Primitive::Int(17)),
            })],
        },
    )
}

#[test]
fn define_variable_with_literal_by_let() {
    common::do_chunk_test(
        "let x = 'abc'",
        Chunk {
            capture: vec![],
            body: vec![Statement::Variable(VariableStatement::Let {
                name: Ident {
                    str: "x",
                    span: (4..5).into(),
                },
                expr: Expression::Primitive(Primitive::String("abc")),
            })],
        },
    )
}

#[test]
fn assign_variable_with_literal() {
    common::do_chunk_test(
        "x = 1.23",
        Chunk {
            capture: vec!["x"],
            body: vec![Statement::Variable(VariableStatement::Assign {
                lhs: Local::Variable {
                    name: Ident {
                        str: "x",
                        span: (0..1).into(),
                    },
                },
                rhs: Expression::Primitive(Primitive::Float(1.23)),
            })],
        },
    )
}

#[test]
fn define_function_without_args_and_body() {
    common::do_chunk_test(
        "func f() end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Variable(VariableStatement::Func {
                name: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (5..6).into(),
                    },
                },
                args: vec![],
                body: Chunk {
                    capture: vec![],
                    body: vec![],
                },
            })],
        },
    );
}

#[test]
fn define_function_with_args_and_body() {
    common::do_chunk_test(
        "func f(a, b) return 'a' end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Variable(VariableStatement::Func {
                name: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (5..6).into(),
                    },
                },
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
                    capture: vec![],
                    body: vec![Statement::Control(ControlStatement::Return {
                        value: Some(Expression::Primitive(Primitive::String("a"))),
                    })],
                },
            })],
        },
    );
}

// Call statement

#[test]
fn call_function_without_args() {
    common::do_chunk_test(
        "f()",
        Chunk {
            capture: vec!["f"],
            body: vec![Statement::Call(CallStatement::Call(Call::Local {
                local: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (0..1).into(),
                    },
                },
                args: vec![],
            }))],
        },
    );
}

#[test]
fn call_function_with_args() {
    common::do_chunk_test(
        "f(1, 'a', true)",
        Chunk {
            capture: vec!["f"],
            body: vec![Statement::Call(CallStatement::Call(Call::Local {
                local: Local::Variable {
                    name: Ident {
                        str: "f",
                        span: (0..1).into(),
                    },
                },
                args: vec![
                    Expression::Primitive(Primitive::Int(1)),
                    Expression::Primitive(Primitive::String("a")),
                    Expression::Primitive(Primitive::Bool(true)),
                ],
            }))],
        },
    );
}

#[test]
fn do_with_no_body() {
    common::do_chunk_test(
        "do end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Call(CallStatement::Do {
                body: Block { body: vec![] },
            })],
        },
    );
}

// Control statement

#[test]
fn if_empty() {
    common::do_chunk_test(
        "if true then end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Control(ControlStatement::If {
                cond: Expression::Primitive(Primitive::Bool(true)),
                body: Block { body: vec![] },
                elifs: vec![],
                else_: None,
            })],
        },
    );
}

#[test]
fn if_elif_else() {
    common::do_chunk_test(
        "if true then return 1 elif false then return 2 else return 3 end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Control(ControlStatement::If {
                cond: Expression::Primitive(Primitive::Bool(true)),
                body: Block {
                    body: vec![Statement::Control(ControlStatement::Return {
                        value: Some(Expression::Primitive(Primitive::Int(1))),
                    })],
                },
                elifs: vec![(
                    Expression::Primitive(Primitive::Bool(false)),
                    Block {
                        body: vec![Statement::Control(ControlStatement::Return {
                            value: Some(Expression::Primitive(Primitive::Int(2))),
                        })],
                    },
                )],
                else_: Some(Block {
                    body: vec![Statement::Control(ControlStatement::Return {
                        value: Some(Expression::Primitive(Primitive::Int(3))),
                    })],
                }),
            })],
        },
    )
}

#[test]
fn for_with_no_step_no_body() {
    common::do_chunk_test(
        "for i = 1, 10 do end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Control(ControlStatement::For {
                value: Local::Variable {
                    name: Ident {
                        str: "i",
                        span: (4..5).into(),
                    },
                },
                start: Expression::Primitive(Primitive::Int(1)),
                stop: Expression::Primitive(Primitive::Int(10)),
                step: None,
                body: Block { body: vec![] },
            })],
        },
    )
}

// #[test]
// fn for_with_nuinus_step() {
//     common::do_chunk_test(
//         "for i = 10, 1, -1 do a = a + i end",
//         Chunk {
//             capture: vec!["a"],
//             body: vec![Statement::Control(ControlStatement::For {
//                 value: Local::Variable {
//                     name: Ident {
//                         str: "i",
//                         span: (4..5).into(),
//                     },
//                 },
//                 start: Expression::Primitive(Primitive::Int(10)),
//                 stop: Expression::Primitive(Primitive::Int(1)),
//                 step: Some(Expression::Primitive(Primitive::Int(-1))),
//                 body: Block {
//                     body: vec![Statement::Variable(VariableStatement::Assign {
//                         lhs: Local::Variable {
//                             name: Ident {
//                                 str: "a",
//                                 span: (12..13).into(),
//                             },
//                         },
//                         rhs: Expression::Binary {
//                             lhs: Box::new(Expression::Local(Local::Variable {
//                                 name: Ident {
//                                     str: "a",
//                                     span: (12..13).into(),
//                                 },
//                             })),
//                             op: BinaryOp::Add,
//                             rhs: Box::new(Expression::Local(Local::Variable {
//                                 name: Ident {
//                                     str: "i",
//                                     span: (16..17).into(),
//                                 },
//                             })),
//                         },
//                     })],
//                 },
//             })],
//         },
//     );
// }

#[test]
fn for_in() {
    common::do_chunk_test(
        "for i in [1, 2, 3] do end",
        Chunk {
            capture: vec![],
            body: vec![Statement::Control(ControlStatement::ForIn {
                value: Local::Variable {
                    name: Ident {
                        str: "i",
                        span: (4..5).into(),
                    },
                },
                iter: Expression::ArrayObject(ArrayObject {
                    elements: vec![
                        Expression::Primitive(Primitive::Int(1)),
                        Expression::Primitive(Primitive::Int(2)),
                        Expression::Primitive(Primitive::Int(3)),
                    ],
                }),
                body: Block { body: vec![] },
            })],
        },
    );
}

#[test]
fn while_() {
    common::do_chunk_test(
        "while ok() do break end",
        Chunk {
            capture: vec!["ok"],
            body: vec![Statement::Control(ControlStatement::While {
                cond: Expression::Call(Call::Local {
                    local: Local::Variable {
                        name: Ident {
                            str: "ok",
                            span: (6..8).into(),
                        },
                    },
                    args: vec![],
                }),
                body: Block {
                    body: vec![Statement::Control(ControlStatement::Break)],
                },
            })],
        },
    );
}

#[test]
fn return_none() {
    common::do_chunk_test(
        "return",
        Chunk {
            capture: vec![],
            body: vec![Statement::Control(ControlStatement::Return { value: None })],
        },
    );
}
