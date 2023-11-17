use super::*;

impl<'node, 'src: 'node> Compilable<'node, 'src> for Expression<'src> {
    fn compile(&'node self, fragment: &mut Fragment<'src>) {
        compile(self, fragment);
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for Box<Expression<'src>> {
    fn compile(&'node self, fragment: &mut Fragment<'src>) {
        compile(self, fragment);
    }
}

fn compile<'node, 'src: 'node>(expr: &'node Expression<'src>, fragment: &mut Fragment<'src>) {
    match expr {
        Expression::Unary {
            op,
            expr: (expr, _),
        } => match op {
            UnaryOp::Neg => {
                fragment.append_compile(expr).append(Code::Unm);
            }
            UnaryOp::Not => {
                // 0: eval expr
                // 1: jump_if_true 4
                // 2: load false
                // 3: jump 5
                // 4: load true
                // 5: ..
                fragment.append_compile(expr).append_many([
                    Code::JumpIfTrue(3),
                    Code::LoadBool(false),
                    Code::Jump(2),
                    Code::LoadBool(true),
                ]);
            }
        },
        Expression::Binary {
            op,
            lhs: (lhs, _),
            rhs: (rhs, _),
        } => match op {
            BinaryOp::Add => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Add);
            }
            BinaryOp::Sub => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Sub);
            }
            BinaryOp::Mul => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Mul);
            }
            BinaryOp::Div => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Div);
            }
            BinaryOp::Mod => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Mod);
            }
            BinaryOp::Pow => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Pow);
            }
            BinaryOp::Eq => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Eq);
            }
            BinaryOp::NotEq => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::NotEq);
            }
            BinaryOp::Less => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Less);
            }
            BinaryOp::LessEq => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::LessEq);
            }
            BinaryOp::Greater => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::Greater);
            }
            BinaryOp::GreaterEq => {
                fragment
                    .append_compile(lhs)
                    .append_compile(rhs)
                    .append(Code::GreaterEq);
            }
            BinaryOp::And => {
                // If lhs is true, then evaluate rhs
                //   0: eval lhs
                //   1: jump_if_false 4
                //   2: eval rhs
                //   3: jump 5
                //   4: push false
                //   5: ...
                let rhs_fragment = Fragment::with_compile(rhs);
                fragment
                    .append_compile(lhs)
                    .append(Code::JumpIfFalse(rhs_fragment.len() as isize + 2))
                    .append_fragment(rhs_fragment)
                    .append(Code::Jump(2))
                    .append(Code::LoadBool(false));
            }
            BinaryOp::Or => {
                // If lhs is false, then evaluate rhs
                //   0: eval lhs
                //   1: jump_if_true 4
                //   2: eval rhs
                //   3: jump 5
                //   4: push true
                //   5: ...
                let rhs_fragment = Fragment::with_compile(rhs);
                fragment
                    .append_compile(lhs)
                    .append(Code::JumpIfTrue(rhs_fragment.len() as isize + 2))
                    .append_fragment(rhs_fragment)
                    .append(Code::Jump(2))
                    .append(Code::LoadBool(true));
            }
        },
        Expression::Ident(ident) => {
            fragment.append(Code::LoadLocal(ident));
        }
        Expression::Primitive(primitive) => match primitive {
            Primitive::Int(x) => {
                fragment.append(Code::LoadInt(*x));
            }
            Primitive::Float(x) => {
                fragment.append(Code::LoadFloat(*x));
            }
            Primitive::String(x) => {
                fragment.append(Code::LoadString(x.clone()));
            }
            Primitive::Bool(x) => {
                fragment.append(Code::LoadBool(*x));
            }
            Primitive::Nil => {
                fragment.append(Code::LoadNil);
            }
        },
        Expression::TableObject(table) => {
            for ((key, _), (value, _)) in table.iter() {
                if let Expression::Ident(ident) = key {
                    fragment
                        .append_compile(value)
                        .append(Code::MakeNamed(ident));
                } else {
                    fragment
                        .append_compile(value)
                        .append_compile(key)
                        .append(Code::MakeExprNamed);
                }
            }
            fragment.append(Code::MakeTable(table.len() as u32));
        }
        Expression::ArrayObject(array) => {
            fragment
                .append_compile_many(array.iter().map(|(expr, _)| expr))
                .append(Code::MakeArray(array.len() as u32));
        }
        Expression::FunctionObject(function) => {
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(function.args.iter().map(|(arg, _)| Code::AddArgument(arg)))
                .append_compile(&function.body)
                .append(Code::EndFuncCreation);
        }
        Expression::Invoke {
            expr: (expr, _),
            args,
        } => {
            fragment
                .append_compile(expr)
                .append_compile_many(args.iter().map(|(expr, _)| expr))
                .append(Code::Call(args.len() as u8));
        }
        Expression::MethodCall {
            expr: (expr, _),
            name: (name, _),
            args,
        } => {
            fragment
                .append_compile(expr)
                .append_compile_many(args.iter().map(|(expr, _)| expr))
                .append(Code::CustomMethod(name, args.len() as u8));
        }
        Expression::IndexAccess {
            expr: (expr, _),
            accesser: (accesser, _),
        } => {
            fragment
                .append_compile(expr)
                .append_compile(accesser)
                .append(Code::GetItem);
        }
        Expression::DotAccess {
            expr: (expr, _),
            accesser: (accesser, _),
        } => {
            fragment
                .append_compile(expr)
                .append(Code::LoadStringAsRef(accesser))
                .append(Code::GetItem);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let fragment = Fragment::with_compile(&Expression::Binary {
            op: BinaryOp::And,
            lhs: (Box::new(Expression::Ident(Ident("a"))), 0..0),
            rhs: (Box::new(Expression::Ident(Ident("b"))), 0..0),
        });
        assert_eq!(
            fragment.into_code(),
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
        let fragment = Fragment::with_compile(&Expression::Binary {
            op: BinaryOp::Or,
            lhs: (Box::new(Expression::Ident(Ident("a"))), 0..0),
            rhs: (Box::new(Expression::Ident(Ident("b"))), 0..0),
        });
        assert_eq!(
            fragment.into_code(),
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
