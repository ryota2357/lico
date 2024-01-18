use super::*;

impl<'node, 'src: 'node> Compilable<'node, 'src> for (Statement<'src>, TextSpan) {
    fn compile(&'node self, fragment: &mut Fragment, context: &mut Context<'src>) -> Result<()> {
        let (statement, span) = self;
        compile(statement, *span, fragment, context)
    }
}

fn compile<'node, 'src: 'node>(
    statement: &'node Statement<'src>,
    span: TextSpan,
    fragment: &mut Fragment,
    context: &mut Context<'src>,
) -> Result<()> {
    match statement {
        // var [name] = [expr]
        Statement::Var {
            name: (name, _),
            expr,
        } => {
            fragment
                .append_compile(expr, context)?
                .append(ICode::MakeLocal);
            context.add_variable(name);
        }

        // func [name]([args])
        //     [body]
        // end
        Statement::Func {
            name: (name, _),
            args,
            body,
        } => {
            // NOTE: `body.captures` is sorted.
            let is_recusive = body
                .captures
                .binary_search_by_key(name, |(name, _)| name)
                .is_ok();
            if is_recusive {
                fragment.append_many([ICode::LoadNil, ICode::MakeLocal]);
                context.add_variable(name);
            }
            util::append_func_creation_fragment(fragment, body, args, context)?;
            if is_recusive {
                let id = context.resolve_variable(name).unwrap();
                fragment.append(ICode::SetLocal(id));
            } else {
                fragment.append(ICode::MakeLocal);
                context.add_variable(name);
            }
        }

        // func [table].[fields]([args])
        //     [body]
        // end
        Statement::FieldFunc {
            table: (_, table_span),
            fields,
            args,
            body,
        } => {
            util::append_func_creation_fragment(fragment, body, args, context)?;
            let mut prev_span_start = table_span.start();
            for (field, field_span) in fields.iter().take(fields.len() - 1) {
                let span = TextSpan::new(prev_span_start, field_span.end());
                prev_span_start = field_span.start();
                fragment
                    .append(ICode::LoadString(field.to_string()))
                    .append(ICode::GetItem(span));
            }
            assert!(!fields.is_empty());
            fragment.append_many([
                // SAFETY: `fields` is not empty because `!fields.is_empty()` is asserted above.
                ICode::LoadString(unsafe { fields.last().unwrap_unchecked() }.0.to_string()),
                ICode::SetItem(span),
            ]);
        }

        // [name] = [expr]
        Statement::Assign {
            name: (name, name_span),
            expr,
        } => {
            fragment.append_compile(expr, context)?;
            let id = context
                .resolve_variable(name)
                .ok_or_else(|| Error::undefined_variable(name.to_string(), *name_span))?;
            fragment.append(ICode::SetLocal(id));
        }

        // [target].[accessor] = [expr]
        Statement::FieldAssign {
            table: target,
            field: accessor,
            expr,
        } => {
            fragment
                .append_compile(expr, context)?
                .append_compile(target, context)?
                .append_compile(accessor, context)?
                .append(ICode::SetItem(span));
        }

        // if [cond] then
        //     [body]
        // elif [elifs.0] then
        //     [elifs.1]
        // else
        //     [else_]
        // end
        Statement::If {
            cond,
            body,
            elifs,
            else_,
        } => {
            // `Set`: [cond]
            //        [jump] // if cond is false, jump to next top of `Set``
            //        [body]
            //        [jump] // [body] is executed, so jump to end of `If`
            //
            // `If` is regarded as array of `Set` (length >= 1) and one `else_`
            //    if `else_` is None, Code::Nop is appended, so `If` always has `else_` block
            //
            // i.e. `If` = `Set`
            //           = `Set`
            //            ...
            //           = `else_`

            let mut new_fragments = {
                // `make_snip` creates [cond] ~ [body]
                let mut make_snip = |cond: &(Expression<'src>, TextSpan), body: &Block<'src>| {
                    let cond_fagment = Fragment::with_compile(cond, context)?;
                    let body_fragment = Fragment::with_compile(body, context)?;
                    let mut fragment = Fragment::new();
                    fragment
                        .append_fragment(cond_fagment)
                        .append(ICode::JumpIfFalse(body_fragment.len() as isize + 2))
                        .append_fragment(body_fragment);
                    Ok(fragment)
                };

                // Applay `make_snip` to (`cond`, `body`) pair, and `elifs`.`
                let mut res = Vec::new();
                res.push(make_snip(cond, body)?);
                for (cond, body) in elifs.iter() {
                    res.push(make_snip(cond, body)?);
                }

                // Append `else_` block
                if let Some(body) = else_ {
                    res.push(Fragment::with_compile(body, context)?);
                } else {
                    res.push(Fragment::with_code(vec![ICode::Nop]));
                }

                res
            };

            // Add last [jump] of `Set`
            let mut jump_dist = new_fragments.last().unwrap().len() + 1;
            for new_frag in new_fragments.iter_mut().rev().skip(1) {
                new_frag.append(ICode::Jump(jump_dist as isize));
                jump_dist += new_frag.len();
            }

            fragment.append_fragment_many(new_fragments);
        }

        // for [value] in [iter] do
        //   [body]
        // end
        //  ↓
        // var <>iter = [iter]->__get_iterator()
        // var [value] = Nil
        // while <>iter->__move_next() do
        //     [value] = <>iter->__current()
        //     [body]
        // end
        // delete [value], <>iter
        Statement::For {
            value: (value, _),
            iter,
            body,
        } => {
            //            0: make_local    <>iter = [iter]->__get_iterator()
            //            1: make_local    [value] = Nil
            // (continue) 2: eval          <>iter->__move_next()
            //            3: jump_if_false 7
            //            4: set_local     [value] = <>iter->__current()
            //            5: eval          [body]
            //            6: jump          2
            //   (break)  7: delete        [value], <>iter (= drop_local 2)
            //            8: ...

            let iter_fragment = Fragment::with_compile(iter, context)?;
            let loop_fragment = {
                let iter_span = iter.1;

                let iter_id = context.add_variable("<>iter");
                let value_id = context.add_variable(value);
                context.begin_loop();
                let body_fragment = Fragment::with_compile(body, context)?; // 6
                let body_fragment_len = body_fragment.len() as isize;
                context.end_loop();
                context.drop_variable(2);

                let mut fragment = Fragment::new();
                fragment
                    .append_many([
                        ICode::CallMethod("__get_iterator".into(), 0, iter_span), // 0
                        ICode::MakeLocal,                                         // |
                        ICode::LoadNil,                                           // 1
                        ICode::MakeLocal,                                         // |
                        ICode::LoadLocal(iter_id),                                // 2
                        ICode::CallMethod("__move_next".into(), 0, iter_span),    // |
                        ICode::JumpIfFalse(3 + body_fragment_len + 2),            // 3
                        ICode::LoadLocal(iter_id),                                // 4
                        ICode::CallMethod("__current".into(), 0, iter_span),      // |
                        ICode::SetLocal(value_id),                                // |
                    ])
                    .append_fragment(body_fragment) // 5
                    .append_many([
                        ICode::Jump(-(body_fragment_len + 6)), //  6
                        ICode::DropLocal(2),                   //  7
                    ]);
                fragment.patch_backward_jump(3); // to 2
                fragment.patch_forward_jump(-1); // to 7
                fragment
            };
            fragment
                .append_fragment(iter_fragment)
                .append_fragment(loop_fragment);
        }

        // while [cond] do
        //     [body]
        // end
        Statement::While { cond, body } => {
            let while_fragment = {
                let cond_fragment = Fragment::with_compile(cond, context)?;
                let cond_fragment_len = cond_fragment.len() as isize;
                let body_fragment = {
                    context.begin_loop();
                    let ret = Fragment::with_compile(body, context)?;
                    context.end_loop();
                    ret
                };
                let body_fragment_len = body_fragment.len() as isize;
                let mut fragment = Fragment::new();
                fragment
                    .append_fragment(cond_fragment)
                    .append(ICode::JumpIfFalse(body_fragment.len() as isize + 2))
                    .append_fragment(body_fragment)
                    .append(ICode::Jump(-(body_fragment_len + 1 + cond_fragment_len)));
                fragment.patch_forward_jump(1);
                fragment.patch_backward_jump(0);
                fragment
            };
            fragment.append_fragment(while_fragment);
        }

        // do
        //     [body]
        // end
        Statement::Do { body } => {
            fragment.append_compile(body, context)?;
        }

        // return [value]
        Statement::Return { value } => {
            if let Some(value) = value {
                fragment.append_compile(value, context)?;
            } else {
                fragment.append(ICode::LoadNil);
            }
            fragment.append(ICode::Return);
        }

        // continue
        Statement::Continue => {
            let drop_count = context.get_loop_vars_count();
            if let Some(drop_count) = drop_count {
                fragment
                    .append(ICode::DropLocal(drop_count))
                    .append_backward_jump();
            } else {
                Err(Error::no_loop_to_continue(span))?;
            }
        }

        // break
        Statement::Break => {
            let drop_count = context.get_loop_vars_count();
            if let Some(drop_count) = drop_count {
                fragment
                    .append(ICode::DropLocal(drop_count))
                    .append_forward_jump();
            } else {
                Err(Error::no_loop_to_break(span))?;
            }
        }

        // [expr]([args])
        Statement::Call { expr, args } => {
            fragment
                .append_compile(expr, context)?
                .append_compile_many(args.iter(), context)?
                .append_many([ICode::Call(args.len() as u8, span), ICode::UnloadTop]);
        }

        // [expr]->[name]([args])
        Statement::MethodCall {
            expr,
            name: (name, span),
            args,
        } => {
            fragment
                .append_compile(expr, context)?
                .append_compile_many(args.iter(), context)?
                .append_many([
                    ICode::CallMethod(name.to_string().into(), args.len() as u8, *span),
                    ICode::UnloadTop,
                ]);
        }

        // @[name]([args])
        Statement::Attribute { name: _, args: _ } => todo!("compile attribute_statement"),

        Statement::Error => {
            panic!("found error");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    pub use pretty_assertions::assert_eq;
    use vm::code::{Code, LocalId};

    #[test]
    fn r#if() {
        let mut context = Context::new();
        context.begin_block();
        context.add_variable("print");
        context.add_variable("a");
        let dummy_span = TextSpan::new(0, 0);
        let statement = (
            Statement::If {
                cond: (Expression::Local("a", dummy_span), dummy_span),
                body: Block(vec![(
                    Statement::Call {
                        expr: (Expression::Local("print", dummy_span), dummy_span),
                        args: vec![],
                    },
                    dummy_span,
                )]),
                elifs: vec![],
                else_: None,
            },
            dummy_span,
        );
        let fragment = Fragment::with_compile(&statement, &mut context);
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal(LocalId(1)), // a
                Code::JumpIfFalse(5),
                Code::LoadLocal(LocalId(0)), // print
                Code::Call(0),
                Code::UnloadTop,
                Code::Jump(2),
                Code::Nop,
            ]
        );
    }

    #[test]
    fn if_else() {
        let mut context = Context::new();
        context.begin_block();
        context.add_variable("print");
        context.add_variable("a");
        let dummy_span = TextSpan::new(0, 0);
        let statement = (
            Statement::If {
                cond: (Expression::Local("a", dummy_span), dummy_span),
                body: Block(vec![(Statement::Return { value: None }, dummy_span)]),
                elifs: vec![],
                else_: Some(Block(vec![(
                    Statement::Call {
                        expr: (Expression::Local("print", dummy_span), dummy_span),
                        args: vec![],
                    },
                    dummy_span,
                )])),
            },
            dummy_span,
        );
        let fragment = Fragment::with_compile(&statement, &mut context);
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal(LocalId(1)), // a
                Code::JumpIfFalse(4),
                Code::LoadNil,
                Code::Return,
                Code::Jump(4),
                Code::LoadLocal(LocalId(0)), // print
                Code::Call(0),
                Code::UnloadTop,
            ]
        );
    }

    #[test]
    fn if_elif() {
        let mut context = Context::new();
        context.begin_block();
        context.add_variable("print");
        context.add_variable("a");
        context.add_variable("b");
        let dummy_span = TextSpan::new(0, 0);
        let statement = (
            Statement::If {
                cond: (Expression::Local("a", dummy_span), dummy_span),
                body: Block(vec![(Statement::Return { value: None }, dummy_span)]),
                elifs: vec![(
                    (Expression::Local("b", dummy_span), dummy_span),
                    Block(vec![(
                        Statement::Call {
                            expr: (Expression::Local("print", dummy_span), dummy_span),
                            args: vec![],
                        },
                        dummy_span,
                    )]),
                )],
                else_: None,
            },
            dummy_span,
        );
        let fragment = Fragment::with_compile(&statement, &mut context);
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal(LocalId(1)), // a
                Code::JumpIfFalse(4),
                Code::LoadNil,
                Code::Return,
                Code::Jump(8),
                Code::LoadLocal(LocalId(2)), // b
                Code::JumpIfFalse(5),
                Code::LoadLocal(LocalId(0)), // print
                Code::Call(0),
                Code::UnloadTop,
                Code::Jump(2),
                Code::Nop,
            ]
        );
    }
}
