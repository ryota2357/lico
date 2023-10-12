mod common;
use parser::tree::*;

fn do_statement_test(src: &str, statement: Statement<'_>) {
    let program = common::parse_program(src);
    let stats = program.body.body;
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0], statement);
}

#[test]
fn arithmetic_op() {
    // ((-1) + ((2 * 3) / 4)) - 5
    do_statement_test(
        "var _ = -1 + 2 * 3 / 4 - 5",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Binary {
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
        }),
    )
}

#[test]
fn arithmetic_op_with_paren() {
    do_statement_test(
        "var _ = (-1 + 2) * 3",
        Statement::Variable(VariableStatement::Var {
            name: Ident {
                str: "_",
                span: (4..5).into(),
            },
            expr: Expression::Binary {
                op: BinaryOp::Mul,
                lhs: Box::new(Expression::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(Expression::Primitive(Primitive::Int(-1))),
                    rhs: Box::new(Expression::Primitive(Primitive::Int(2))),
                }),
                rhs: Box::new(Expression::Primitive(Primitive::Int(3))),
            },
        }),
    )
}
