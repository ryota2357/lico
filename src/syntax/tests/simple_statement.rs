mod common;

use syntax::tree::node::*;

// Variable statement

#[test]
fn define_variable_with_literal_by_var() {
    common::do_tree_test(
        "var x = 17",
        Block(vec![(
            Statement::Variable(VariableStatement::Var {
                lhs: (Local::Variable("x"), (4..5).into()),
                rhs: (Expression::Primitive(Primitive::Int(17)), (8..10).into()),
            }),
            (0..10).into(),
        )]),
    );
}

#[test]
fn define_variable_with_literal_by_let() {
    common::do_tree_test(
        "let x = 'abc'",
        Block(vec![(
            Statement::Variable(VariableStatement::Let {
                lhs: (Local::Variable("x"), (4..5).into()),
                rhs: (
                    Expression::Primitive(Primitive::String("abc")),
                    (8..13).into(),
                ),
            }),
            (0..13).into(),
        )]),
    );
}

#[test]
fn assign_variable_with_literal() {
    common::do_tree_test(
        "x = 1.23",
        Block(vec![(
            Statement::Variable(VariableStatement::Assign {
                lhs: (Local::Variable("x"), (0..1).into()),
                rhs: (Expression::Primitive(Primitive::Float(1.23)), (4..8).into()),
            }),
            (0..8).into(),
        )]),
    );
}

// Function statement

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn define_function_without_args_and_body() {
    common::do_tree_test(
        "func f() end",
        Block(vec![(
            Statement::Function(FunctionStatement::Define {
                name: (Local::Variable("f"), (5..6).into()),
                args: (vec![], (7..7).into()),
                body: (Block(vec![]), (9..8).into()), // FIXME: Empty range
            }),
            (0..12).into(),
        )]),
    );
}

#[test]
fn define_function_without_args() {
    common::do_tree_test(
        "func f() return 1 end",
        Block(vec![(
            Statement::Function(FunctionStatement::Define {
                name: (Local::Variable("f"), (5..6).into()),
                args: (vec![], (7..7).into()),
                body: (
                    Block(vec![(
                        Statement::Control(ControlStatement::Return(Some((
                            Expression::Primitive(Primitive::Int(1)),
                            (16..17).into(),
                        )))),
                        (9..17).into(),
                    )]),
                    (9..17).into(),
                ),
            }),
            (0..21).into(),
        )]),
    );
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn define_function_without_body() {
    common::do_tree_test(
        "func f(x, y) end",
        Block(vec![(
            Statement::Function(FunctionStatement::Define {
                name: (Local::Variable("f"), (5..6).into()),
                args: (
                    vec![
                        (Expression::Local(Local::Variable("x")), (7..8).into()),
                        (Expression::Local(Local::Variable("y")), (10..11).into()),
                    ],
                    (7..11).into(),
                ),
                body: (Block(vec![]), (13..12).into()), // FIXME: Empty range
            }),
            (0..16).into(),
        )]),
    );
}

#[test]
fn define_function_with_all_and_trailing_comma() {
    common::do_tree_test(
        "func f(x, y,) return 'a' end",
        Block(vec![(
            Statement::Function(FunctionStatement::Define {
                name: (Local::Variable("f"), (5..6).into()),
                args: (
                    vec![
                        (Expression::Local(Local::Variable("x")), (7..8).into()),
                        (Expression::Local(Local::Variable("y")), (10..11).into()),
                    ],
                    (7..12).into(),
                ),
                body: (
                    Block(vec![(
                        Statement::Control(ControlStatement::Return(Some((
                            Expression::Primitive(Primitive::String("a")),
                            (21..24).into(),
                        )))),
                        (14..24).into(),
                    )]),
                    (14..24).into(),
                ),
            }),
            (0..28).into(),
        )]),
    );
}

#[test]
fn call_function_without_args() {
    common::do_tree_test(
        "f()",
        Block(vec![(
            Statement::Function(FunctionStatement::Call {
                name: (Callable::Local(Local::Variable("f")), (0..1).into()),
                args: (vec![], (2..2).into()),
            }),
            (0..3).into(),
        )]),
    );
}

#[test]
fn call_function_with_args() {
    common::do_tree_test(
        "f(1, 'a', true)",
        Block(vec![(
            Statement::Function(FunctionStatement::Call {
                name: (Callable::Local(Local::Variable("f")), (0..1).into()),
                args: (
                    vec![
                        (Expression::Primitive(Primitive::Int(1)), (2..3).into()),
                        (Expression::Primitive(Primitive::String("a")), (5..8).into()),
                        (
                            Expression::Primitive(Primitive::Bool(true)),
                            (10..14).into(),
                        ),
                    ],
                    (2..14).into(),
                ),
            }),
            (0..15).into(),
        )]),
    );
}

#[test]
fn call_function_trailing_comma() {
    common::do_tree_test(
        "f(0,)",
        Block(vec![(
            Statement::Function(FunctionStatement::Call {
                name: (Callable::Local(Local::Variable("f")), (0..1).into()),
                args: (
                    vec![(Expression::Primitive(Primitive::Int(0)), (2..3).into())],
                    (2..4).into(),
                ),
            }),
            (0..5).into(),
        )]),
    );
}

// Control statement

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn if_empty() {
    common::do_tree_test(
        "if true then end",
        Block(vec![(
            Statement::Control(ControlStatement::If {
                cond: (Expression::Primitive(Primitive::Bool(true)), (3..7).into()),
                body: (Block(vec![]), (13..12).into()), // FIXME: Empty range
                elifs: vec![],
                else_: None,
            }),
            (0..16).into(),
        )]),
    );
}

#[test]
fn if_else() {
    common::do_tree_test(
        "if true then return 1 else return 2 end",
        Block(vec![(
            Statement::Control(ControlStatement::If {
                cond: (Expression::Primitive(Primitive::Bool(true)), (3..7).into()),
                body: (
                    Block(vec![(
                        Statement::Control(ControlStatement::Return(Some((
                            Expression::Primitive(Primitive::Int(1)),
                            (20..21).into(),
                        )))),
                        (13..21).into(),
                    )]),
                    (13..21).into(),
                ),
                elifs: vec![],
                else_: Some((
                    Block(vec![(
                        Statement::Control(ControlStatement::Return(Some((
                            Expression::Primitive(Primitive::Int(2)),
                            (34..35).into(),
                        )))),
                        (27..35).into(),
                    )]),
                    (27..35).into(),
                )),
            }),
            (0..39).into(),
        )]),
    );
}

#[test]
fn if_elif() {
    common::do_tree_test(
        "if false then return 1 elif true then return 2 end",
        Block(vec![(
            Statement::Control(ControlStatement::If {
                cond: (Expression::Primitive(Primitive::Bool(false)), (3..8).into()),
                body: (
                    Block(vec![(
                        Statement::Control(ControlStatement::Return(Some((
                            Expression::Primitive(Primitive::Int(1)),
                            (21..22).into(),
                        )))),
                        (14..22).into(),
                    )]),
                    (14..22).into(),
                ),
                elifs: vec![(
                    (
                        Expression::Primitive(Primitive::Bool(true)),
                        (28..32).into(),
                    ),
                    (
                        Block(vec![(
                            Statement::Control(ControlStatement::Return(Some((
                                Expression::Primitive(Primitive::Int(2)),
                                (45..46).into(),
                            )))),
                            (38..46).into(),
                        )]),
                        (38..46).into(),
                    ),
                )],
                else_: None,
            }),
            (0..50).into(),
        )]),
    );
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn for_with_no_step_no_body() {
    common::do_tree_test(
        "for i = 1, 10 do end",
        Block(vec![(
            Statement::Control(ControlStatement::For {
                value: (Local::Variable("i"), (4..5).into()),
                start: (Expression::Primitive(Primitive::Int(1)), (8..9).into()),
                end: (Expression::Primitive(Primitive::Int(10)), (11..13).into()),
                step: None,
                body: (Block(vec![]), (17..16).into()), // FIXME: Empty range
            }),
            (0..20).into(),
        )]),
    );
}

#[test]
fn for_with_muinus_step() {
    common::do_tree_test(
        "for i = 10, 1, -1 do a = a + i end",
        Block(vec![(
            Statement::Control(ControlStatement::For {
                value: (Local::Variable("i"), (4..5).into()),
                start: (Expression::Primitive(Primitive::Int(10)), (8..10).into()),
                end: (Expression::Primitive(Primitive::Int(1)), (12..13).into()),
                step: Some((Expression::Primitive(Primitive::Int(-1)), (15..17).into())),
                body: (
                    Block(vec![(
                        Statement::Variable(VariableStatement::Assign {
                            lhs: (Local::Variable("a"), (21..22).into()),
                            rhs: (
                                Expression::Binary {
                                    op: (BinaryOp::Add, (27..28).into()),
                                    lhs: (
                                        Box::new(Expression::Local(Local::Variable("a"))),
                                        (25..26).into(),
                                    ),
                                    rhs: (
                                        Box::new(Expression::Local(Local::Variable("i"))),
                                        (29..30).into(),
                                    ),
                                },
                                (25..30).into(),
                            ),
                        }),
                        (21..30).into(),
                    )]),
                    (21..30).into(),
                ),
            }),
            (0..34).into(),
        )]),
    );
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn for_in_without_body() {
    common::do_tree_test(
        "for x in it() do end",
        Block(vec![(
            Statement::Control(ControlStatement::ForIn {
                value: (Local::Variable("x"), (4..5).into()),
                iter: (
                    Expression::FunctionCall {
                        func: (Callable::Local(Local::Variable("it")), (9..11).into()),
                        args: (vec![], (12..12).into()),
                    },
                    (9..13).into(),
                ),
                body: (Block(vec![]), (17..16).into()), // FIXME: Empty range
            }),
            (0..20).into(),
        )]),
    );
}

#[test]
fn for_in() {
    common::do_tree_test(
        "for x in a do p(x) end",
        Block(vec![(
            Statement::Control(ControlStatement::ForIn {
                value: (Local::Variable("x"), (4..5).into()),
                iter: (Expression::Local(Local::Variable("a")), (9..10).into()),
                body: (
                    Block(vec![(
                        Statement::Function(FunctionStatement::Call {
                            name: (Callable::Local(Local::Variable("p")), (14..15).into()),
                            args: (
                                vec![(Expression::Local(Local::Variable("x")), (16..17).into())],
                                (16..17).into(),
                            ),
                        }),
                        (14..18).into(),
                    )]),
                    (14..18).into(),
                ),
            }),
            (0..22).into(),
        )]),
    );
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn while_without_body() {
    common::do_tree_test(
        "while ok() do end",
        Block(vec![(
            Statement::Control(ControlStatement::While {
                cond: (
                    Expression::FunctionCall {
                        func: (Callable::Local(Local::Variable("ok")), (6..8).into()),
                        args: (vec![], (9..9).into()),
                    },
                    (6..10).into(),
                ),
                body: (Block(vec![]), (14..13).into()), // FIXME: Empty range
            }),
            (0..17).into(),
        )]),
    );
}

#[test]
fn while_() {
    common::do_tree_test(
        "while true do break end",
        Block(vec![(
            Statement::Control(ControlStatement::While {
                cond: (Expression::Primitive(Primitive::Bool(true)), (6..10).into()),
                body: (
                    Block(vec![(
                        Statement::Control(ControlStatement::Break),
                        (14..19).into(),
                    )]),
                    (14..19).into(),
                ),
            }),
            (0..23).into(),
        )]),
    );
}

#[test]
fn return_none() {
    common::do_tree_test(
        "return",
        Block(vec![(
            Statement::Control(ControlStatement::Return(None)),
            (0..6).into(),
        )]),
    );
}
