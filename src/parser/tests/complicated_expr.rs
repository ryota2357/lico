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
            expr: Box::new(Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (3..4).into(),
            }))),
            args: vec![],
        },
    );
    do_expr_test(
        "(((f(1))))",
        Expression::Invoke {
            expr: Box::new(Expression::Local(Local::Ident(Ident {
                str: "f",
                span: (5..6).into(),
            }))),
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
        Expression::Local(Local::Ident(Ident {
            str: "a",
            span: (3..4).into(),
        })),
    );
    do_expr_test(
        "(((a)))",
        Expression::Local(Local::Ident(Ident {
            str: "a",
            span: (5..6).into(),
        })),
    );
}
