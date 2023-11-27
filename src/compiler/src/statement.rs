use super::*;

impl<'node, 'src: 'node> ContextCompilable<'node, 'src> for (Statement<'src>, Span) {
    fn compile(&'node self, fragment: &mut Fragment<'src>, context: &mut Context) -> Result<()> {
        let (statement, span) = self;
        match statement {
            Statement::Control(statement) => control_statement(statement, span, fragment, context)?,
            Statement::Attribute(statement) => attribute_statement(statement, fragment, context)?,
            Statement::Variable(statement) => variable_statement(statement, fragment, context)?,
            Statement::Call(statement) => call_statement(statement, fragment)?,
        }
        Ok(())
    }
}

fn control_statement<'node, 'src: 'node>(
    statement: &'node ControlStatement<'src>,
    span: &Span,
    fragment: &mut Fragment<'src>,
    context: &mut Context,
) -> Result<()> {
    match statement {
        ControlStatement::If {
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
                let mut make_snip = |cond: &(Expression<'src>, Span), body: &Block<'src>| {
                    let body_fragment = Fragment::with_compile_with_context(body, context)?;
                    let mut fragment = Fragment::new();
                    fragment
                        .append_compile(cond)?
                        .append(Code::JumpIfFalse(body_fragment.len() as isize + 2))
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
                    res.push(Fragment::with_compile_with_context(body, context)?);
                } else {
                    res.push(Fragment::with_code(vec![Code::Nop]));
                }

                res
            };

            // Add last [jump] of `Set`
            let mut jump_dist = new_fragments.last().unwrap().len() + 1;
            for new_frag in new_fragments.iter_mut().rev().skip(1) {
                new_frag.append(Code::Jump(jump_dist as isize));
                jump_dist += new_frag.len();
            }

            fragment.append_fragment_many(new_fragments);
            Ok(())
        }
        ControlStatement::For {
            value: (value, _),
            iter,
            body,
        } => {
            // var <>iter = [iter]->__get_iterator()
            // while <>iter->__move_next() do
            //     var [value] = <>iter->__current()
            // end
            // delete <>iter
            //
            //            0: make_local    <>iter = [iter]->__get_iterator()
            //            1: jump          3
            // (continue) 2: delete        [value] (= drop_local 1)
            //            3: eval          <>iter->__move_next()
            //            4: jump_if_false 10
            //            5: make_local    [value] = <>iter->__current()
            //            6: eval          [body]
            //            7: jump          2
            //   (break)  8: delete        [value], <>iter (= drop_local 2)
            //        |   9: jump          11
            //           10: delete        <>iter (= drop_local 1)
            //           11: ...

            let iter_fragment = Fragment::with_compile(iter)?;

            let loop_fragment = {
                context.start_loop_section();
                let body_fragment = Fragment::with_compile_with_context(body, context)?;
                let body_fragment_len = body_fragment.len() as isize;
                context.end_loop_section();

                let mut fragment = Fragment::new();
                fragment
                    .append_many([
                        Code::CallMethod("__get_iterator", 0),        // 0
                        Code::MakeLocal("<>iter"),                    // |
                        Code::Jump(2),                                // 1
                        Code::DropLocal(1),                           // 2
                        Code::LoadLocal("<>iter"),                    // 3
                        Code::CallMethod("__move_next", 0),           // |
                        Code::JumpIfFalse(3 + body_fragment_len + 4), // 4
                        Code::LoadLocal("<>iter"),                    // 5
                        Code::CallMethod("__current", 0),             // |
                        Code::MakeLocal(value),                       // |
                    ])
                    .append_fragment(body_fragment) // 6
                    .append_many([
                        Code::Jump(-(body_fragment_len + 7)), //  7
                        Code::DropLocal(2),                   //  8
                        Code::Jump(2),                        //  9
                        Code::DropLocal(1),                   // 10
                    ]);
                fragment.patch_backward_jump(3); // to 2
                fragment.patch_forward_jump(-2); // to 8
                fragment
            };

            fragment
                .append_fragment(iter_fragment)
                .append_fragment(loop_fragment);
            Ok(())
        }
        ControlStatement::While { cond, body } => {
            let while_fragment = {
                let cond_fragment = Fragment::with_compile(cond)?;
                let cond_fragment_len = cond_fragment.len() as isize;

                let body_fragment = {
                    context.start_loop_section();
                    let ret = Fragment::with_compile_with_context(body, context)?;
                    context.end_loop_section();
                    ret
                };
                let body_fragment_len = body_fragment.len() as isize;

                let mut fragment = Fragment::new();
                fragment
                    .append_fragment(cond_fragment)
                    .append(Code::JumpIfFalse(body_fragment_len + 2))
                    .append_fragment(body_fragment)
                    .append(Code::Jump(-(body_fragment_len + 1 + cond_fragment_len)));
                fragment.patch_forward_jump(1);
                fragment.patch_backward_jump(0);

                fragment
            };
            fragment.append_fragment(while_fragment);
            Ok(())
        }
        ControlStatement::Do { body } => {
            fragment.append_compile_with_context(body, context)?;
            Ok(())
        }
        ControlStatement::Return { value } => {
            if let Some(value) = value {
                fragment.append_compile(value)?;
            } else {
                fragment.append(Code::LoadNil);
            }
            fragment.append(Code::Return);
            Ok(())
        }
        ControlStatement::Continue => {
            let drop_count = context.get_loop_local_count();
            if let Some(drop_count) = drop_count {
                fragment.append(Code::DropLocal(drop_count));
                fragment.append_backward_jump();
            } else {
                Err(Error::no_loop_to_continue(span.clone()))?;
            }
            Ok(())
        }
        ControlStatement::Break => {
            let drop_count = context.get_loop_local_count();
            if let Some(drop_count) = drop_count {
                fragment.append(Code::DropLocal(drop_count));
                fragment.append_forward_jump();
            } else {
                Err(Error::no_loop_to_break(span.clone()))?;
            }
            Ok(())
        }
    }
}

fn attribute_statement<'node, 'src: 'node>(
    _statement: &'node AttributeStatement<'src>,
    _fragment: &mut Fragment<'src>,
    _context: &mut Context,
) -> Result<()> {
    unimplemented!("attribute_statement")
}

fn variable_statement<'node, 'src: 'node>(
    statement: &'node VariableStatement<'src>,
    fragment: &mut Fragment<'src>,
    context: &mut Context,
) -> Result<()> {
    match statement {
        VariableStatement::Var {
            name: (name, _),
            expr,
        } => {
            fragment.append_compile(expr)?.append(Code::MakeLocal(name));
            context.inc_local_count();
            Ok(())
        }
        VariableStatement::Func {
            name: (name, _),
            args,
            body,
        } => {
            let recusive = body.captures.contains(name);
            if recusive {
                fragment.append_many([Code::LoadNil, Code::MakeLocal(name)]);
            }
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(args.iter().map(|(arg, _)| Code::AddArgument(arg)))
                .append_compile(body)?
                .append(Code::EndFuncCreation)
                .append(if recusive {
                    Code::SetLocal(name)
                } else {
                    Code::MakeLocal(name)
                });
            context.inc_local_count();
            Ok(())
        }
        VariableStatement::FieldFunc {
            table: (table, _),
            fields,
            args,
            body,
        } => {
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(args.iter().map(|(arg, _)| Code::AddArgument(arg)))
                .append_compile(body)?
                .append(Code::EndFuncCreation)
                .append(Code::LoadLocal(table))
                .append_many(
                    fields
                        .iter()
                        .take(fields.len() - 1)
                        .flat_map(|(field, _)| [Code::LoadStringAsRef(field), Code::GetItem]),
                )
                .append(Code::LoadStringAsRef(
                    fields.last().map(|(field, _)| field).unwrap(),
                ))
                .append(Code::SetItem);
            Ok(())
        }
        VariableStatement::Assign {
            name: (name, _),
            accesser,
            expr,
        } => {
            fragment.append_compile(expr)?;
            if accesser.is_empty() {
                fragment.append(Code::SetLocal(name));
            } else {
                fragment.append(Code::LoadLocal(name));
                for acc in accesser.iter().take(accesser.len() - 1) {
                    fragment.append_compile(acc)?.append(Code::GetItem);
                }
                fragment
                    .append_compile(accesser.last().unwrap())?
                    .append(Code::SetItem);
            }
            Ok(())
        }
    }
}

fn call_statement<'node, 'src: 'node>(
    statement: &'node CallStatement<'src>,
    fragment: &mut Fragment<'src>,
) -> Result<()> {
    match statement {
        CallStatement::Invoke { expr, args } => {
            fragment
                .append_compile(expr)?
                .append_compile_many(args.iter())?
                .append_many([Code::Call(args.len() as u8), Code::UnloadTop]);
            Ok(())
        }
        CallStatement::MethodCall {
            expr,
            name: (name, _),
            args,
        } => {
            fragment
                .append_compile(expr)?
                .append_compile_many(args.iter())?
                .append_many([Code::CallMethod(name, args.len() as u8), Code::UnloadTop]);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#if() {
        let statement = (
            Statement::Control(ControlStatement::If {
                cond: (Expression::Ident(Ident("a")), 0..0),
                body: Block(vec![(
                    Statement::Call(CallStatement::Invoke {
                        expr: (Expression::Ident(Ident("print")), 0..0),
                        args: vec![],
                    }),
                    0..0,
                )]),
                elifs: vec![],
                else_: None,
            }),
            0..0,
        );
        let fragment = Fragment::with_compile_with_context(&statement, &mut Context::new());
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal("a"),
                Code::JumpIfFalse(5),
                Code::LoadLocal("print"),
                Code::Call(0),
                Code::UnloadTop,
                Code::Jump(2),
                Code::Nop,
            ]
        );
    }

    #[test]
    fn if_else() {
        let statement = (
            Statement::Control(ControlStatement::If {
                cond: (Expression::Ident(Ident("a")), 0..0),
                body: Block(vec![(
                    Statement::Control(ControlStatement::Return { value: None }),
                    0..0,
                )]),
                elifs: vec![],
                else_: Some(Block(vec![(
                    Statement::Call(CallStatement::Invoke {
                        expr: (Expression::Ident(Ident("print")), 0..0),
                        args: vec![],
                    }),
                    0..0,
                )])),
            }),
            0..0,
        );
        let fragment = Fragment::with_compile_with_context(&statement, &mut Context::new());
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal("a"),
                Code::JumpIfFalse(4),
                Code::LoadNil,
                Code::Return,
                Code::Jump(4),
                Code::LoadLocal("print"),
                Code::Call(0),
                Code::UnloadTop,
            ]
        );
    }

    #[test]
    fn if_elif() {
        let statement = (
            Statement::Control(ControlStatement::If {
                cond: (Expression::Ident(Ident("a")), 0..0),
                body: Block(vec![(
                    Statement::Control(ControlStatement::Return { value: None }),
                    0..0,
                )]),
                elifs: vec![(
                    (Expression::Ident(Ident("b")), 0..0),
                    Block(vec![(
                        Statement::Call(CallStatement::Invoke {
                            expr: (Expression::Ident(Ident("print")), 0..0),
                            args: vec![],
                        }),
                        0..0,
                    )]),
                )],
                else_: None,
            }),
            0..0,
        );
        let fragment = Fragment::with_compile_with_context(&statement, &mut Context::new());
        assert_eq!(
            fragment.unwrap().into_code(),
            vec![
                Code::LoadLocal("a"),
                Code::JumpIfFalse(4),
                Code::LoadNil,
                Code::Return,
                Code::Jump(8),
                Code::LoadLocal("b"),
                Code::JumpIfFalse(5),
                Code::LoadLocal("print"),
                Code::Call(0),
                Code::UnloadTop,
                Code::Jump(2),
                Code::Nop,
            ]
        );
    }
}
