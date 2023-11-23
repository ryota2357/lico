use super::*;

impl<'node, 'src: 'node> Compilable<'node, 'src> for (Expression<'src>, Span) {
    fn compile(&'node self, fragment: &mut Fragment<'src>) -> Result<()> {
        let (expr, _) = self;
        compile(expr, fragment)
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for (Box<Expression<'src>>, Span) {
    fn compile(&'node self, fragment: &mut Fragment<'src>) -> Result<()> {
        let (expr, _) = self;
        compile(expr, fragment)
    }
}

fn compile<'node, 'src: 'node>(
    expr: &'node Expression<'src>,
    fragment: &mut Fragment<'src>,
) -> Result<()> {
    match expr {
        Expression::Unary { op, expr } => match op {
            UnaryOp::Neg => {
                fragment.append_compile(expr)?.append(Code::Unm);
                Ok(())
            }
            UnaryOp::Not => {
                // 0: eval expr
                // 1: jump_if_true 4
                // 2: load false
                // 3: jump 5
                // 4: load true
                // 5: ..
                fragment.append_compile(expr)?.append_many([
                    Code::JumpIfTrue(3),
                    Code::LoadBool(false),
                    Code::Jump(2),
                    Code::LoadBool(true),
                ]);
                Ok(())
            }
        },
        Expression::Binary { op, lhs, rhs } => match op {
            BinaryOp::Add => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Add);
                Ok(())
            }
            BinaryOp::Sub => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Sub);
                Ok(())
            }
            BinaryOp::Mul => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Mul);
                Ok(())
            }
            BinaryOp::Div => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Div);
                Ok(())
            }
            BinaryOp::Mod => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Mod);
                Ok(())
            }
            BinaryOp::Pow => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Pow);
                Ok(())
            }
            BinaryOp::Eq => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Eq);
                Ok(())
            }
            BinaryOp::NotEq => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::NotEq);
                Ok(())
            }
            BinaryOp::Less => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Less);
                Ok(())
            }
            BinaryOp::LessEq => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::LessEq);
                Ok(())
            }
            BinaryOp::Greater => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Greater);
                Ok(())
            }
            BinaryOp::GreaterEq => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::GreaterEq);
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
                let rhs_fragment = Fragment::with_compile(rhs)?;
                fragment
                    .append_compile(lhs)?
                    .append(Code::JumpIfFalse(rhs_fragment.len() as isize + 2))
                    .append_fragment(rhs_fragment)
                    .append(Code::Jump(2))
                    .append(Code::LoadBool(false));
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
                let rhs_fragment = Fragment::with_compile(rhs)?;
                fragment
                    .append_compile(lhs)?
                    .append(Code::JumpIfTrue(rhs_fragment.len() as isize + 2))
                    .append_fragment(rhs_fragment)
                    .append(Code::Jump(2))
                    .append(Code::LoadBool(true));
                Ok(())
            }
            BinaryOp::Concat => {
                fragment
                    .append_compile(lhs)?
                    .append_compile(rhs)?
                    .append(Code::Concat);
                Ok(())
            }
        },
        Expression::Ident(ident) => {
            fragment.append(Code::LoadLocal(ident));
            Ok(())
        }
        Expression::Primitive(primitive) => match primitive {
            Primitive::Int(x) => {
                fragment.append(Code::LoadInt(*x));
                Ok(())
            }
            Primitive::Float(x) => {
                fragment.append(Code::LoadFloat(*x));
                Ok(())
            }
            Primitive::String(x) => {
                fragment.append(Code::LoadString(x.clone()));
                Ok(())
            }
            Primitive::Bool(x) => {
                fragment.append(Code::LoadBool(*x));
                Ok(())
            }
            Primitive::Nil => {
                fragment.append(Code::LoadNil);
                Ok(())
            }
        },
        Expression::TableObject(table) => {
            for (key, value) in table.iter() {
                if let (Expression::Ident(ident), _) = key {
                    fragment
                        .append_compile(value)?
                        .append(Code::MakeNamed(ident));
                } else {
                    fragment
                        .append_compile(value)?
                        .append_compile(key)?
                        .append(Code::MakeExprNamed);
                }
            }
            fragment.append(Code::MakeTable(table.len() as u32));
            Ok(())
        }
        Expression::ArrayObject(array) => {
            fragment
                .append_compile_many(array.iter())?
                .append(Code::MakeArray(array.len() as u32));
            Ok(())
        }
        Expression::FunctionObject(function) => {
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(function.args.iter().map(|(arg, _)| Code::AddArgument(arg)))
                .append_compile(&function.body)?
                .append(Code::EndFuncCreation);
            Ok(())
        }
        Expression::Invoke { expr, args } => {
            fragment
                .append_compile(expr)?
                .append_compile_many(args.iter())?
                .append(Code::Call(args.len() as u8));
            Ok(())
        }
        Expression::MethodCall {
            expr,
            name: (name, _),
            args,
        } => {
            fragment
                .append_compile(expr)?
                .append_compile_many(args.iter())?
                .append(Code::CustomMethod(name, args.len() as u8));
            Ok(())
        }
        Expression::IndexAccess { expr, accesser } => {
            fragment
                .append_compile(expr)?
                .append_compile(accesser)?
                .append(Code::GetItem);
            Ok(())
        }
        Expression::DotAccess {
            expr,
            accesser: (accesser, _),
        } => {
            fragment
                .append_compile(expr)?
                .append(Code::LoadStringAsRef(accesser))
                .append(Code::GetItem);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let fragment = Fragment::with_compile(&(
            Expression::Binary {
                op: BinaryOp::And,
                lhs: (Box::new(Expression::Ident(Ident("a"))), 0..0),
                rhs: (Box::new(Expression::Ident(Ident("b"))), 0..0),
            },
            0..0,
        ));
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal("a"),
                Code::JumpIfFalse(3),
                Code::LoadLocal("b"),
                Code::Jump(2),
                Code::LoadBool(false)
            ]
        );
    }

    #[test]
    fn or() {
        let fragment = Fragment::with_compile(&(
            Expression::Binary {
                op: BinaryOp::Or,
                lhs: (Box::new(Expression::Ident(Ident("a"))), 0..0),
                rhs: (Box::new(Expression::Ident(Ident("b"))), 0..0),
            },
            0..0,
        ));
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal("a"),
                Code::JumpIfTrue(3),
                Code::LoadLocal("b"),
                Code::Jump(2),
                Code::LoadBool(true)
            ]
        );
    }
}
