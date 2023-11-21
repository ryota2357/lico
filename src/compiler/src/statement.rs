use super::*;

impl<'node, 'src: 'node> ContextCompilable<'node, 'src> for Statement<'src> {
    fn compile(&'node self, fragment: &mut Fragment<'src>, context: &mut Context) -> Result<()> {
        match self {
            Statement::Control(statement) => control_statement(statement, fragment, context)?,
            Statement::Attribute(statement) => attribute_statement(statement, fragment, context)?,
            Statement::Variable(statement) => variable_statement(statement, fragment, context)?,
            Statement::Call(statement) => call_statement(statement, fragment)?,
        }
        Ok(())
    }
}

fn control_statement<'node, 'src: 'node>(
    statement: &'node ControlStatement<'src>,
    fragment: &mut Fragment<'src>,
    context: &mut Context,
) -> Result<()> {
    match statement {
        ControlStatement::If {
            cond: (cond, _),
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
                let mut make_snip = |cond: &'node Expression<'src>, body: &'node Block<'src>| {
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
                for ((cond, _), body) in elifs.iter() {
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
            iter: (iter, _),
            body,
        } => {
            //            0: make_local    <>iter = [iter]->__get_iterator()
            //            1: eval          <>iter->__move_next()
            //            2: jump_if_false 11
            //            3: jump          6
            // (continue) 4: eval          <>iter->__move_next()
            //            5: jump_if_false 9
            //            6: make_local    [value] = <>iter->__current()
            //            7: eval          [body]
            //            8: jump          4
            //    (break) 9: delete        [value], <>iter (== drop_local 2)
            //           10: jump          12
            //           11: delete        <>iter  (== drop_local 1)
            //           12: ...

            let iter_fragment = Fragment::with_compile(iter)?;

            let loop_fragment = {
                context.start_loop_section();
                let body_fragment = Fragment::with_compile_with_context(body, context)?;
                let body_fragment_len = body_fragment.len() as isize;
                context.end_loop_section();

                let mut fragment = Fragment::new();
                fragment
                    .append_many([
                        Code::CustomMethod("__get_iterator", 0),      // 0
                        Code::MakeLocal("<>iter"),                    // |
                        Code::LoadLocal("<>iter"),                    // 1
                        Code::CustomMethod("__move_next", 0),         // |
                        Code::JumpIfFalse(7 + body_fragment_len + 4), // 2
                        Code::Jump(4),                                // 3
                        Code::LoadLocal("<>iter"),                    // 4
                        Code::CustomMethod("__move_next", 0),         // |
                        Code::JumpIfFalse(3 + body_fragment_len + 2), // 5
                        Code::LoadLocal("<>iter"),                    // 6
                        Code::CustomMethod("__current", 0),           // |
                        Code::MakeLocal(value),                       // |
                    ])
                    .append_fragment(body_fragment)
                    .append_many([
                        Code::Jump(-(body_fragment_len + 6)), //  8
                        Code::DropLocal(2),                   //  9
                        Code::Jump(2),                        // 10
                        Code::DropLocal(1),                   // 11
                    ]);
                fragment.patch_backward_jump(6); // to 4
                fragment.patch_forward_jump(-2); // to 9
                fragment
            };

            fragment
                .append_fragment(iter_fragment)
                .append_fragment(loop_fragment);
            Ok(())
        }
        ControlStatement::While {
            cond: (cond, _),
            body,
        } => {
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
            if let Some((value, _)) = value {
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
                // Err(Error::no_loop_to_continue(...))?;
                panic!("Cannot continue outside of loop");
            }
            Ok(())
        }
        ControlStatement::Break => {
            let drop_count = context.get_loop_local_count();
            if let Some(drop_count) = drop_count {
                fragment.append(Code::DropLocal(drop_count));
                fragment.append_forward_jump();
            } else {
                // Err(Error::no_loop_to_break(...))?;
                panic!("Cannot break outside of loop");
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
            expr: (expr, _),
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
            expr: (expr, _),
        } => {
            fragment.append_compile(expr)?;
            if accesser.is_empty() {
                fragment.append(Code::SetLocal(name));
            } else {
                fragment.append(Code::LoadLocal(name));
                for (acc, _) in accesser.iter().take(accesser.len() - 1) {
                    fragment.append_compile(acc)?.append(Code::GetItem);
                }
                fragment
                    .append_compile(accesser.last().map(|(acc, _)| acc).unwrap())?
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
        CallStatement::Invoke {
            expr: (expr, _),
            args,
        } => {
            fragment
                .append_compile(expr)?
                .append_compile_many(args.iter().map(|(expr, _)| expr))?
                .append_many([Code::Call(args.len() as u8), Code::UnloadTop]);
            Ok(())
        }
        CallStatement::MethodCall {
            expr: (expr, _),
            name: (name, _),
            args,
        } => {
            fragment
                .append_compile(expr)?
                .append_compile_many(args.iter().map(|(expr, _)| expr))?
                .append_many([Code::CustomMethod(name, args.len() as u8), Code::UnloadTop]);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#if() {
        let statement = Statement::Control(ControlStatement::If {
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
        });
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
        let statement = Statement::Control(ControlStatement::If {
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
        });
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
        let statement = Statement::Control(ControlStatement::If {
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
        });
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
