use super::*;

impl<'a> Compilable<'a> for Expression<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>) {
        compile(self, fragment);
    }
}

impl<'a> Compilable<'a> for Box<Expression<'a>> {
    fn compile(&self, fragment: &mut Fragment<'a>) {
        compile(self, fragment);
    }
}

fn compile<'a>(expr: &Expression<'a>, fragment: &mut Fragment<'a>) {
    match expr {
        Expression::Unary { op, expr } => match op {
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
        Expression::Binary { op, lhs, rhs } => match op {
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
                    .append(Code::JumpIfFalse(rhs_fragment.len() as isize + 3))
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
                    .append(Code::JumpIfTrue(rhs_fragment.len() as isize + 3))
                    .append_fragment(rhs_fragment)
                    .append(Code::Jump(2))
                    .append(Code::LoadBool(true));
            }
        },
        Expression::Ident(ident) => {
            fragment.append(Code::LoadLocal(ident.str));
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
            for (key, value) in table.iter() {
                if let Expression::Ident(ident) = key {
                    fragment
                        .append_compile(value)
                        .append(Code::MakeNamed(ident.str));
                } else {
                    fragment
                        .append_compile(value)
                        .append_compile(key)
                        .append(Code::MakeExprNamed);
                }
            }
            fragment.append(Code::MakeTable(table.key_values.len() as u32));
        }
        Expression::ArrayObject(array) => {
            fragment
                .append_compile_many(array)
                .append(Code::MakeArray(array.len() as u32));
        }
        Expression::FunctionObject(function) => {
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(function.args.iter().map(|arg| Code::AddArgument(arg.str)))
                .append_compile(&function.body)
                .append(Code::EndFuncCreation);
        }
        Expression::Invoke { expr, args } => {
            fragment
                .append_compile(expr)
                .append_compile_many(args)
                .append(Code::Call(args.len() as u8));
        }
        Expression::MethodCall { expr, name, args } => {
            fragment
                .append_compile(expr)
                .append_compile_many(args)
                .append(Code::CustomMethod(name.str, args.len() as u8));
        }
        Expression::IndexAccess { expr, accesser } => {
            fragment
                .append_compile(expr)
                .append_compile(accesser)
                .append(Code::GetItem);
        }
        Expression::DotAccess { expr, accesser } => {
            fragment
                .append_compile(expr)
                .append(Code::LoadStringAsRef(accesser.str))
                .append(Code::GetItem);
        }
    }
}
