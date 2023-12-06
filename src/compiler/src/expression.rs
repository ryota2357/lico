use super::*;
use parser::tree::*;

impl<'node, 'src: 'node> Compilable<'node, 'src> for (Expression<'src>, Span) {
    fn compile(&'node self, fragment: &mut Fragment, context: &mut Context<'src>) -> Result<()> {
        let (expr, span) = self;
        compile(expr, span.clone(), fragment, context)
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for (Box<Expression<'src>>, Span) {
    fn compile(&'node self, fragment: &mut Fragment, context: &mut Context<'src>) -> Result<()> {
        let (expr, span) = self;
        compile(expr, span.clone(), fragment, context)
    }
}

fn compile<'node, 'src: 'node>(
    expr: &'node Expression<'src>,
    span: Span,
    fragment: &mut Fragment,
    context: &mut Context<'src>,
) -> Result<()> {
    match expr {
        Expression::Unary { op, expr } => match op {
            UnaryOp::Neg => {
                fragment
                    .append_compile(expr, context)?
                    .append(ICode::Unm(span));
                Ok(())
            }
            UnaryOp::Not => {
                // 0: eval expr
                // 1: jump_if_true 4
                // 2: load false
                // 3: jump 5
                // 4: load true
                // 5: ..
                fragment.append_compile(expr, context)?.append_many([
                    ICode::JumpIfTrue(3),
                    ICode::LoadBool(false),
                    ICode::Jump(2),
                    ICode::LoadBool(true),
                ]);
                Ok(())
            }
        },
        Expression::Binary { op, lhs, rhs } => match op {
            BinaryOp::Add => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Add(span));
                Ok(())
            }
            BinaryOp::Sub => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Sub(span));
                Ok(())
            }
            BinaryOp::Mul => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Mul(span));
                Ok(())
            }
            BinaryOp::Div => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Div(span));
                Ok(())
            }
            BinaryOp::Mod => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Mod(span));
                Ok(())
            }
            BinaryOp::Pow => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Pow(span));
                Ok(())
            }
            BinaryOp::Eq => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Eq(span));
                Ok(())
            }
            BinaryOp::NotEq => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::NotEq(span));
                Ok(())
            }
            BinaryOp::Less => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Less(span));
                Ok(())
            }
            BinaryOp::LessEq => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::LessEq(span));
                Ok(())
            }
            BinaryOp::Greater => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Greater(span));
                Ok(())
            }
            BinaryOp::GreaterEq => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::GreaterEq(span));
                Ok(())
            }
            BinaryOp::And => {
                // If lhs is true, then evaluate rhs
                //   0: eval lhs
                //   1: jump_if_false 4
                //   2: eval rhs
                //   3: jump 5
                //   4: push false
                //   5: ...
                let lhs_fragment = Fragment::with_compile(lhs, context)?;
                let rhs_fragment = Fragment::with_compile(rhs, context)?;
                fragment
                    .append_fragment(lhs_fragment)
                    .append(ICode::JumpIfFalse(rhs_fragment.len() as isize + 2))
                    .append_fragment(rhs_fragment)
                    .append_many([ICode::Jump(2), ICode::LoadBool(false)]);
                Ok(())
            }
            BinaryOp::Or => {
                // If lhs is false, then evaluate rhs
                //   0: eval lhs
                //   1: jump_if_true 4
                //   2: eval rhs
                //   3: jump 5
                //   4: push true
                //   5: ...
                let lhs_fragment = Fragment::with_compile(lhs, context)?;
                let rhs_fragment = Fragment::with_compile(rhs, context)?;
                fragment
                    .append_fragment(lhs_fragment)
                    .append(ICode::JumpIfTrue(rhs_fragment.len() as isize + 2))
                    .append_fragment(rhs_fragment)
                    .append_many([ICode::Jump(2), ICode::LoadBool(true)]);
                Ok(())
            }
            BinaryOp::Concat => {
                fragment
                    .append_compile(lhs, context)?
                    .append_compile(rhs, context)?
                    .append(ICode::Concat(span));
                Ok(())
            }
        },
        Expression::Ident(Ident(name, _)) => {
            let id = context
                .resolve_variable(name)
                .ok_or_else(|| Error::undefined_variable(name.to_string(), span.clone()))?;
            fragment.append(ICode::LoadLocal(id));
            Ok(())
        }
        Expression::Primitive(primitive) => match primitive {
            Primitive::Int(x) => {
                fragment.append(ICode::LoadInt(*x));
                Ok(())
            }
            Primitive::Float(x) => {
                fragment.append(ICode::LoadFloat(*x));
                Ok(())
            }
            Primitive::String(x) => {
                fragment.append(ICode::LoadString(x.clone()));
                Ok(())
            }
            Primitive::Bool(x) => {
                fragment.append(ICode::LoadBool(*x));
                Ok(())
            }
            Primitive::Nil => {
                fragment.append(ICode::LoadNil);
                Ok(())
            }
        },
        Expression::TableObject(table) => {
            for (key, value) in table.iter() {
                fragment
                    .append_compile(value, context)?
                    .append_compile(key, context)?
                    .append(ICode::MakeNamed);
            }
            fragment.append(ICode::MakeTable(table.len() as u32));
            Ok(())
        }
        Expression::ArrayObject(array) => {
            fragment
                .append_compile_many(array.iter(), context)?
                .append(ICode::MakeArray(array.len() as u32));
            Ok(())
        }
        Expression::FunctionObject(function) => {
            util::append_func_creation_fragment(fragment, &function.body, &function.args, context)?;
            Ok(())
        }
        Expression::Invoke { expr, args } => {
            fragment
                .append_compile(expr, context)?
                .append_compile_many(args.iter(), context)?
                .append(ICode::Call(args.len() as u8, span));
            Ok(())
        }
        Expression::MethodCall {
            expr,
            name: Ident(name, _),
            args,
        } => {
            fragment
                .append_compile(expr, context)?
                .append_compile_many(args.iter(), context)?
                .append(ICode::CallMethod(
                    name.to_string().into(),
                    args.len() as u8,
                    span,
                ));
            Ok(())
        }
        Expression::IndexAccess { expr, accesser } => {
            fragment
                .append_compile(expr, context)?
                .append_compile(accesser, context)?
                .append(ICode::GetItem(span));
            Ok(())
        }
        Expression::DotAccess {
            expr,
            accesser: Ident(accesser, _),
        } => {
            fragment
                .append_compile(expr, context)?
                .append(ICode::LoadString(accesser.to_string()))
                .append(ICode::GetItem(span));
            Ok(())
        }
        Expression::Error => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vm::code::{Code, LocalId};

    #[test]
    fn and() {
        let mut context = Context::new();
        context.begin_block();
        context.add_variable("a");
        context.add_variable("b");
        let fragment = Fragment::with_compile(
            &(
                Expression::Binary {
                    op: BinaryOp::And,
                    lhs: (Box::new(Expression::Ident(Ident("a", 0..0))), 0..0),
                    rhs: (Box::new(Expression::Ident(Ident("b", 0..0))), 0..0),
                },
                0..0,
            ),
            &mut context,
        );
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal(LocalId(0)),
                Code::JumpIfFalse(3),
                Code::LoadLocal(LocalId(1)),
                Code::Jump(2),
                Code::LoadBool(false)
            ]
        );
    }

    #[test]
    fn or() {
        let mut context = Context::new();
        context.begin_block();
        context.add_variable("a");
        context.add_variable("b");
        let fragment = Fragment::with_compile(
            &(
                Expression::Binary {
                    op: BinaryOp::Or,
                    lhs: (Box::new(Expression::Ident(Ident("a", 0..0))), 0..0),
                    rhs: (Box::new(Expression::Ident(Ident("b", 0..0))), 0..0),
                },
                0..0,
            ),
            &mut context,
        );
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal(LocalId(0)),
                Code::JumpIfTrue(3),
                Code::LoadLocal(LocalId(1)),
                Code::Jump(2),
                Code::LoadBool(true)
            ]
        );
    }
}
