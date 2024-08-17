use foundation::object::UString;

use super::*;

impl<'node, 'src: 'node> Compilable<'node, 'src> for ir::ValueKey {
    fn compile(&'node self, fragment: &mut Fragment, ctx: &mut Context<'src>) {
        let Some((_, value)) = ctx.strage.get(self) else {
            panic!("[BUG] Missing value must be resolved at caller side");
        };
        fragment.append_compile(&value, ctx);
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for &ir::Value {
    fn compile(&'node self, fragment: &mut Fragment, ctx: &mut Context<'src>) {
        compile(self, fragment, ctx);
    }
}

fn compile<'node, 'src: 'node>(
    value: &ir::Value,
    fragment: &mut Fragment,
    ctx: &mut Context<'src>,
) {
    use ir::Value;
    use ICodeSource::*;

    match value {
        // 0: eval           [condition]
        // 1: jump_if_false  4
        // 2: eval           [then]
        // 3: jump           5
        // 4: eval           [else]
        // 5: ...
        Value::Branch {
            condition,
            then,
            then_tail,
            else_,
            else_tail,
        } => {
            fragment.append_compile(condition, ctx);
            let (then_fragment, then_len) = {
                let m = ctx.start_block();
                let mut fragment = Fragment::with_compile(then, ctx);
                match ctx.strage.get(then_tail) {
                    Some((_, tail)) => fragment.append_compile(&tail, ctx),
                    None => fragment.append(LoadNilObject),
                };
                fragment.append(DropLocal(ctx.get_block_local_count()));
                m.finish(ctx);
                let len = fragment.len() as isize;
                (fragment, len)
            };
            let (else_fragment, else_len) = {
                let m = ctx.start_block();
                let mut fragment = Fragment::with_compile(else_, ctx);
                match ctx.strage.get(else_tail) {
                    Some((_, tail)) => fragment.append_compile(&tail, ctx),
                    None => fragment.append(LoadNilObject),
                };
                fragment.append(DropLocal(ctx.get_block_local_count()));
                m.finish(ctx);
                let len = fragment.len() as isize;
                (fragment, len)
            };
            fragment
                .append(JumpIfFalse(then_len + 2))
                .append_fragment(then_fragment)
                .append(Jump(else_len + 1))
                .append_fragment(else_fragment);
        }

        Value::Prefix { op, value } => {
            fragment.append_compile(value, ctx).append(match op {
                ir::PrefixOp::Plus(t) => Unp(t.text_range()),
                ir::PrefixOp::Minus(t) => Unm(t.text_range()),
                ir::PrefixOp::Not(t) => Not(t.text_range()),
                ir::PrefixOp::BitNot(t) => BitNot(t.text_range()),
                ir::PrefixOp::Missing => {
                    panic!("Missing prefix operator, this error must be resolved upstream.")
                }
            });
        }

        Value::Binary { op, lhs, rhs } => {
            match op {
                //   0: eval lhs
                //   1: jump_if_false 4
                //   2: eval rhs
                //   3: jump 5
                //   4: push false
                //   5: ...
                ir::BinaryOp::And(_) => {
                    let lhs_fragment = Fragment::with_compile(lhs, ctx);
                    let rhs_fragment = Fragment::with_compile(rhs, ctx);
                    fragment
                        .append_fragment(lhs_fragment)
                        .append(JumpIfFalse(rhs_fragment.len() as isize + 2))
                        .append_fragment(rhs_fragment)
                        .append_many([Jump(2), LoadBoolObject(false)]);
                    return;
                }
                //   0: eval lhs
                //   1: jump_if_true 4
                //   2: eval rhs
                //   3: jump 5
                //   4: push true
                //   5: ...
                ir::BinaryOp::Or(_) => {
                    let lhs_fragment = Fragment::with_compile(lhs, ctx);
                    let rhs_fragment = Fragment::with_compile(rhs, ctx);
                    fragment
                        .append_fragment(lhs_fragment)
                        .append(JumpIfTrue(rhs_fragment.len() as isize + 2))
                        .append_fragment(rhs_fragment)
                        .append_many([Jump(2), LoadBoolObject(true)]);
                    return;
                }
                _ => {}
            }
            fragment
                .append_compile(lhs, ctx)
                .append_compile(rhs, ctx)
                .append(match op {
                    ir::BinaryOp::Add(t) => Add(t.text_range()),
                    ir::BinaryOp::Sub(t) => Sub(t.text_range()),
                    ir::BinaryOp::Mul(t) => Mul(t.text_range()),
                    ir::BinaryOp::Div(t) => Div(t.text_range()),
                    ir::BinaryOp::Mod(t) => Mod(t.text_range()),
                    ir::BinaryOp::Shl(t) => ShiftL(t.text_range()),
                    ir::BinaryOp::Shr(t) => ShiftR(t.text_range()),
                    ir::BinaryOp::Concat(t) => Concat(t.text_range()),
                    ir::BinaryOp::Eq(t) => Eq(t.text_range()),
                    ir::BinaryOp::Ne(t) => NotEq(t.text_range()),
                    ir::BinaryOp::Lt(t) => Less(t.text_range()),
                    ir::BinaryOp::Le(t) => LessEq(t.text_range()),
                    ir::BinaryOp::Gt(t) => Greater(t.text_range()),
                    ir::BinaryOp::Ge(t) => GreaterEq(t.text_range()),
                    ir::BinaryOp::And(_) => unreachable!(),
                    ir::BinaryOp::Or(_) => unreachable!(),
                    ir::BinaryOp::BitAnd(t) => BitAnd(t.text_range()),
                    ir::BinaryOp::BitOr(t) => BitOr(t.text_range()),
                    ir::BinaryOp::BitXor(t) => BitXor(t.text_range()),
                    ir::BinaryOp::Assign(_) => panic!(
                        "Invalid binary operator `Assign`, this error must be resolved upstream."
                    ),
                    ir::BinaryOp::Missing => {
                        panic!("Missing binary operator, this error must be resolved upstream.")
                    }
                });
        }

        Value::Call { value, args } => {
            let (calee_syntax, value) = ctx.strage.get(value).unwrap();
            fragment.append_compile(&value, ctx);
            assert!(
                args.len() <= u8::MAX as usize,
                "Number of arguments greater than u8::MAX is not supported."
            );
            let mut args_range = Vec::with_capacity(args.len());
            for (syntax, arg) in ctx.strage.get(args) {
                fragment.append_compile(&arg, ctx);
                args_range.push(syntax.text_range());
            }
            fragment.append(Call(
                args.len() as u8,
                calee_syntax.text_range(),
                args_range.into_boxed_slice(),
            ));
        }

        Value::Index { value, index } => {
            let (index_syntax, index_value) = ctx.strage.get(index).unwrap();
            fragment
                .append_compile(value, ctx)
                .append_compile(&index_value, ctx)
                .append(GetItem(index_syntax.text_range()));
        }

        Value::Field { value, name } => {
            let (field_syntax, field_string) = ctx.strage.get(name).unwrap();
            fragment
                .append_compile(value, ctx)
                .append(LoadStringObject(UString::from(field_string.clone())))
                .append(GetItem(field_syntax.text_range()));
        }

        Value::MethodCall { value, name, args } => {
            let mut ranges = Vec::with_capacity(args.len() + 1);

            let (value_syntax, value) = ctx.strage.get(value).unwrap();
            fragment.append_compile(&value, ctx);
            ranges.push(value_syntax.text_range());

            let (name_syntax, name_string) = ctx.strage.get(name).unwrap();
            ranges.push(name_syntax.text_range());

            assert!(
                args.len() <= u8::MAX as usize,
                "Number of arguments greater than u8::MAX is not supported."
            );
            for (syntax, arg) in ctx.strage.get(args) {
                fragment.append_compile(&arg, ctx);
                ranges.push(syntax.text_range());
            }

            fragment.append(CallMethod(
                args.len() as u8,
                UString::from(name_string.clone()),
                ranges.into_boxed_slice(),
            ));
        }

        Value::Block { effects, tail } => {
            let m = ctx.start_block();
            fragment.append_compile(effects, ctx);
            if let Some((_, tail)) = ctx.strage.get(tail) {
                fragment.append_compile(&tail, ctx);
            } else {
                fragment.append(LoadNilObject);
            }
            m.finish(ctx);
        }

        Value::Local { name } => {
            let name_str = ctx.strage.get(name).unwrap().1.text();
            let local_id = ctx.resolve_local(name_str);
            fragment.append(LoadLocal(local_id));
        }

        Value::Int(x) => {
            fragment.append(LoadIntObject(*x));
        }

        Value::Float(x) => {
            fragment.append(LoadFloatObject(*x));
        }

        Value::String(x) => {
            fragment.append(LoadStringObject(x.clone()));
        }

        Value::Bool(x) => {
            fragment.append(LoadBoolObject(*x));
        }

        Value::Nil => {
            fragment.append(LoadNilObject);
        }

        Value::Function(func) => {
            compile_utils::compile_function(None, func, fragment, ctx);
        }

        Value::Array { elements } => {
            for (_, element) in ctx.strage.get(elements) {
                fragment.append_compile(&element, ctx);
            }
            fragment.append(MakeArray(elements.len()));
        }

        Value::Table { fields } => {
            let mut key_ranges = Vec::new();
            for (key, value) in fields {
                match key {
                    ir::TableKeyName::Value(v) => {
                        let Some((syntax, value)) = ctx.strage.get(v) else {
                            panic!("Missing table key, this error must be resolved upstream.");
                        };
                        fragment.append_compile(&value, ctx);
                        key_ranges.push(Some(syntax.text_range()));
                    }
                    ir::TableKeyName::String(s) => {
                        let key_str = ctx.strage.get(s).unwrap().0.text();
                        fragment.append(LoadStringObject(UString::from(key_str)));
                        key_ranges.push(None);
                    }
                }
                fragment.append_compile(value, ctx);
            }
            fragment.append(MakeTable(fields.len(), key_ranges.into_boxed_slice()));
        }
    }
}
