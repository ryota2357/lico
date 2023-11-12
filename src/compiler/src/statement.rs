use super::*;

impl<'a> ContextCompilable<'a> for Statement<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>, context: &mut Context) {
        match self {
            Statement::Control(statement) => control_statement(statement, fragment, context),
            Statement::Attribute(statement) => attribute_statement(statement, fragment, context),
            Statement::Variable(statement) => variable_statement(statement, fragment, context),
            Statement::Call(statement) => call_statement(statement, fragment),
        };
    }
}

fn control_statement<'a>(
    statement: &ControlStatement<'a>,
    fragment: &mut Fragment<'a>,
    context: &mut Context,
) {
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
                let mut make_snip = |cond: &Expression<'a>, body: &Block<'a>| {
                    let body_fragment = Fragment::with_compile_with_context(body, context);
                    let mut fragment = Fragment::new();
                    fragment
                        .append_compile(cond)
                        .append(Code::JumpIfFalse(body_fragment.len() as isize + 2))
                        .append_fragment(body_fragment);
                    fragment
                };

                // Applay `make_snip` to (`cond`, `body`) pair, and `elifs`.`
                let mut res = Vec::new();
                res.push(make_snip(cond, body));
                for (cond, body) in elifs.iter() {
                    res.push(make_snip(cond, body));
                }

                // Append `else_` block
                if let Some(body) = else_ {
                    res.push(Fragment::with_compile_with_context(body, context));
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
        }
        ControlStatement::For { value, iter, body } => {
            //            0: make_local    <>iter = [iter]->__getIterator()
            //            1: eval          <>iter->__moveNext()
            //            2: jump_if_false 11
            //            3: jump          6
            // (continue) 4: eval          <>iter->__moveNext()
            //            5: jump_if_false 9
            //            6: make_local    [value] = <>iter->__getCurrent()
            //            7: eval          [body]
            //            8: jump          4
            //    (break) 9: delete        [value], <>iter (== drop_local 2)
            //           10: jump          12
            //           11: delete        <>iter  (== drop_local 1)
            //           12: ...

            let iter_fragment = Fragment::with_compile(iter);

            let loop_fragment = {
                context.start_loop_section();
                let body_fragment = Fragment::with_compile_with_context(body, context);
                let body_fragment_len = body_fragment.len() as isize;
                context.end_loop_section();

                let mut fragment = Fragment::new();
                fragment
                    .append_many([
                        Code::CustomMethod("__getIterator", 0),       // 0
                        Code::MakeLocal("<>iter"),                    // |
                        Code::LoadLocal("<>iter"),                    // 1
                        Code::CustomMethod("__moveNext", 0),          // |
                        Code::JumpIfFalse(7 + body_fragment_len + 4), // 2
                        Code::Jump(4),                                // 3
                        Code::LoadLocal("<>iter"),                    // 4
                        Code::CustomMethod("__moveNext", 0),          // |
                        Code::JumpIfFalse(3 + body_fragment_len + 2), // 5
                        Code::LoadLocal("<>iter"),                    // 6
                        Code::CustomMethod("__getCurrent", 0),        // |
                        Code::MakeLocal(value.str),                   // |
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
        }
        ControlStatement::While { cond, body } => {
            let while_fragment = {
                let cond_fragment = Fragment::with_compile(cond);
                let cond_fragment_len = cond_fragment.len() as isize;

                let body_fragment = {
                    context.start_loop_section();
                    let ret = Fragment::with_compile_with_context(body, context);
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
        }
        ControlStatement::Do { body } => {
            fragment.append_compile_with_context(body, context);
        }
        ControlStatement::Return { value } => {
            if let Some(value) = value {
                fragment.append_compile(value);
            } else {
                fragment.append(Code::LoadNil);
            }
            fragment.append(Code::Return);
        }
        ControlStatement::Continue => {
            let drop_count = context.get_loop_local_count();
            if let Some(drop_count) = drop_count {
                fragment.append(Code::DropLocal(drop_count));
                fragment.append_backward_jump();
            } else {
                panic!("Cannot continue outside of loop");
            }
        }
        ControlStatement::Break => {
            let drop_count = context.get_loop_local_count();
            if let Some(drop_count) = drop_count {
                fragment.append(Code::DropLocal(drop_count));
                fragment.append_forward_jump();
            } else {
                panic!("Cannot break outside of loop");
            }
        }
    }
}

fn attribute_statement<'a>(
    _statement: &AttributeStatement<'a>,
    _fragment: &mut Fragment<'a>,
    _context: &mut Context,
) {
    unimplemented!("attribute_statement")
}

fn variable_statement<'a>(
    statement: &VariableStatement<'a>,
    fragment: &mut Fragment<'a>,
    context: &mut Context,
) {
    match statement {
        VariableStatement::Var { name, expr } => {
            fragment
                .append_compile(expr)
                .append(Code::MakeLocal(name.str));
            context.inc_local_count();
        }
        VariableStatement::Func { name, args, body } => {
            let recusive = body.captures.contains(&name.str);
            if recusive {
                fragment.append_many([Code::LoadNil, Code::MakeLocal(name.str)]);
            }
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(args.iter().map(|arg| Code::AddArgument(arg.str)))
                .append_compile(body)
                .append(Code::EndFuncCreation)
                .append(if recusive {
                    Code::SetLocal(name.str)
                } else {
                    Code::MakeLocal(name.str)
                });
            context.inc_local_count();
        }
        VariableStatement::FieldFunc {
            table,
            fields,
            args,
            body,
        } => {
            fragment
                .append(Code::BeginFuncCreation)
                .append_many(args.iter().map(|arg| Code::AddArgument(arg.str)))
                .append_compile(body)
                .append(Code::EndFuncCreation)
                .append(Code::LoadLocal(table.str))
                .append_many(
                    fields
                        .iter()
                        .take(fields.len() - 1)
                        .flat_map(|field| [Code::LoadStringAsRef(field.str), Code::GetItem]),
                )
                .append(Code::LoadStringAsRef(fields.last().unwrap().str))
                .append(Code::SetItem);
        }
        VariableStatement::Assign {
            name,
            accesser,
            expr,
        } => {
            fragment.append_compile(expr);
            if accesser.is_empty() {
                fragment.append(Code::SetLocal(name.str));
            } else {
                for acc in accesser.iter().take(accesser.len() - 1) {
                    fragment.append_compile(acc).append(Code::GetItem);
                }
                fragment
                    .append_compile(accesser.last().unwrap())
                    .append(Code::SetItem);
            }
        }
    }
}

fn call_statement<'a>(statement: &CallStatement<'a>, fragment: &mut Fragment<'a>) {
    match statement {
        CallStatement::Invoke { expr, args } => {
            fragment
                .append_compile(expr)
                .append_compile_many(args)
                .append_many([Code::Call(args.len() as u8), Code::UnloadTop]);
        }
        CallStatement::MethodCall { expr, name, args } => {
            fragment
                .append_compile(expr)
                .append_compile_many(args)
                .append_many([
                    Code::CustomMethod(name.str, args.len() as u8),
                    Code::UnloadTop,
                ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#if() {
        let statement = Statement::Control(ControlStatement::If {
            cond: Expression::Ident(Ident {
                str: "a",
                span: (0..0).into(),
            }),
            body: Block {
                body: vec![Statement::Call(CallStatement::Invoke {
                    expr: Expression::Ident(Ident {
                        str: "print",
                        span: (0..0).into(),
                    }),
                    args: vec![],
                })],
            },
            elifs: vec![],
            else_: None,
        });
        let fragment = Fragment::with_compile_with_context(&statement, &mut Context::new());
        assert_eq!(
            fragment.into_code(),
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
            cond: Expression::Ident(Ident {
                str: "a",
                span: (0..0).into(),
            }),
            body: Block {
                body: vec![Statement::Control(ControlStatement::Return { value: None })],
            },
            elifs: vec![],
            else_: Some(Block {
                body: vec![Statement::Call(CallStatement::Invoke {
                    expr: Expression::Ident(Ident {
                        str: "print",
                        span: (0..0).into(),
                    }),
                    args: vec![],
                })],
            }),
        });
        let fragment = Fragment::with_compile_with_context(&statement, &mut Context::new());
        assert_eq!(
            fragment.into_code(),
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
            cond: Expression::Ident(Ident {
                str: "a",
                span: (0..0).into(),
            }),
            body: Block {
                body: vec![Statement::Control(ControlStatement::Return { value: None })],
            },
            elifs: vec![(
                Expression::Ident(Ident {
                    str: "b",
                    span: (0..0).into(),
                }),
                Block {
                    body: vec![Statement::Call(CallStatement::Invoke {
                        expr: Expression::Ident(Ident {
                            str: "print",
                            span: (0..0).into(),
                        }),
                        args: vec![],
                    })],
                },
            )],
            else_: None,
        });
        let fragment = Fragment::with_compile_with_context(&statement, &mut Context::new());
        assert_eq!(
            fragment.into_code(),
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
