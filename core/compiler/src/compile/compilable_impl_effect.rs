use super::*;
use foundation::object::UString;

impl<'node, 'src: 'node> Compilable<'node, 'src> for ir::EffectsKey {
    fn compile(&'node self, fragment: &mut Fragment, ctx: &mut Context<'src>) {
        for (_, effect) in ctx.strage.get(self) {
            fragment.append_compile(&effect, ctx);
        }
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for &ir::Effect {
    fn compile(&'node self, fragment: &mut Fragment, ctx: &mut Context<'src>) {
        compile(self, fragment, ctx);
    }
}

fn compile<'node, 'src: 'node>(
    effect: &ir::Effect,
    fragment: &mut Fragment,
    ctx: &mut Context<'src>,
) {
    use ir::Effect;
    use ICodeSource::*;

    match effect {
        Effect::MakeLocal { name, value } => {
            fragment.append_compile(value, ctx).append(StoreNewLocal);
            let name_str = ctx.strage.get(name).unwrap().1.text();
            ctx.add_local(name_str);
        }

        Effect::MakeFunc { name, func } => {
            let name_str = ctx.strage.get(name).unwrap().1.text();
            compile_utils::compile_function(Some(name_str), func, fragment, ctx);
            fragment.append(StoreNewLocal);
            ctx.add_local(name_str);
        }

        Effect::SetLocal { local, value } => {
            fragment.append_compile(value, ctx);
            let name_str = ctx.strage.get(local).unwrap().1.text();
            let local_id = ctx.resolve_local(name_str);
            fragment.append_many([StoreLocal(local_id)]);
        }

        Effect::SetIndex {
            target,
            index,
            value,
        } => {
            let (index_syntax, index) = ctx.strage.get(index).unwrap();
            fragment
                .append_compile(value, ctx)
                .append_compile(target, ctx)
                .append_compile(&index, ctx)
                .append(SetItem(index_syntax.text_range()));
        }

        Effect::SetField {
            target,
            field,
            value,
        } => {
            let (field_syntax, field_string) = ctx.strage.get(field).unwrap();
            fragment
                .append_compile(value, ctx)
                .append_compile(target, ctx)
                .append_many([
                    LoadStringObject(UString::from(field_string.clone())),
                    SetItem(field_syntax.text_range()),
                ]);
        }

        Effect::SetFieldFunc { table, path, func } => {
            let table_symbol = ctx.strage.get(table).unwrap().1;
            let path_len = path.len();
            let path_iter = ctx.strage.get(path);

            compile_utils::compile_function(None, func, fragment, ctx);
            fragment
                .append(LoadLocal(ctx.resolve_local(table_symbol.text())))
                .append_many(path_iter.enumerate().flat_map(|(i, (syntax, field))| {
                    [
                        LoadStringObject(UString::from(field.clone())),
                        if i == path_len - 1 {
                            GetItem(syntax.text_range())
                        } else {
                            SetItem(syntax.text_range())
                        },
                    ]
                }));
        }

        Effect::SetMethod {
            table,
            path,
            name,
            func,
        } => {
            let table_symbol = ctx.strage.get(table).unwrap().1;
            let path_iter = ctx.strage.get(path);
            let (name_syntax, name_string) = ctx.strage.get(name).unwrap();

            compile_utils::compile_function(None, func, fragment, ctx);
            fragment
                .append(LoadLocal(ctx.resolve_local(table_symbol.text())))
                .append_many(path_iter.flat_map(|(syntax, field)| {
                    [
                        LoadStringObject(UString::from(field.clone())),
                        GetItem(syntax.text_range()),
                    ]
                }))
                .append(SetMethod(
                    UString::from(name_string.clone()),
                    name_syntax.text_range(),
                ));
        }

        // 0: eval           [condition]
        // 1: jump_if_false  4
        // 2: eval           [then]
        // 3: jump           5
        // 4: eval           [else]
        // 5: ...
        Effect::Branch {
            condition,
            then,
            else_,
        } => {
            fragment.append_compile(condition, ctx);
            let (then_fragment, then_len) = {
                let m = ctx.start_block();
                let mut fragment = Fragment::with_compile(then, ctx);
                fragment.append(DropLocal(ctx.get_block_local_count()));
                m.finish(ctx);
                let len = fragment.len() as isize;
                (fragment, len)
            };
            let (else_fragment, else_len) = {
                let m = ctx.start_block();
                let mut fragment = Fragment::with_compile(else_, ctx);
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

        //            0: make_local    <>iter = [iterable]->__get_iterator()
        //            1: make_local    [variable] = nil
        // (continue) 2: eval          <>iter->__move_next()
        //            3: jump_if_false 7
        //            4: set_local     [variable] = <>iter->__current()
        //            5: eval          [effects]
        //            6: jump          2
        //    (break) 7: delete        <>iter, [variable]
        //            8: ...
        Effect::LoopFor {
            variable,
            iterable,
            effects,
        } => {
            fragment.append_compile(iterable, ctx);
            let loop_fragment = {
                let iter_id = ctx.add_local("<>iter");
                let variable_id = ctx.add_local(ctx.strage.get(variable).unwrap().1.text());

                let m = ctx.start_loop();
                let (effects_fragment, effects_len) = {
                    let m = ctx.start_block();
                    let mut fragment = Fragment::with_compile(effects, ctx);
                    fragment.append(DropLocal(ctx.get_block_local_count()));
                    m.finish(ctx);
                    let len = fragment.len() as isize;
                    (fragment, len)
                };
                let mut fragment = Fragment::new();
                fragment
                    .append_many([
                        GetIter,                      // 0
                        StoreNewLocal,                // |
                        LoadNilObject,                // 1
                        StoreNewLocal,                // |
                        LoadLocal(iter_id),           // 2
                        IterMoveNext,                 // |
                        JumpIfFalse(effects_len + 2), // 3
                        LoadLocal(iter_id),           // 4
                        IterCurrent,                  // |
                        StoreLocal(variable_id),      // |
                    ])
                    .append_fragment(effects_fragment) // 5
                    .append_many([
                        Jump(-effects_len - 6), // 6
                        DropLocal(2),           // 7
                    ]);
                fragment.patch_backward_jump(4); // to 2
                fragment.patch_forward_jump(-1); // to 7
                m.finish(ctx);
                ctx.drop_local(2);
                fragment
            };
            fragment.append_fragment(loop_fragment);
        }

        Effect::LoopWhile { condition, effects } => {
            let while_fragment = {
                let cond_fragment = Fragment::with_compile(condition, ctx);
                let cond_len = cond_fragment.len() as isize;
                let (effects_fragment, effects_len) = {
                    let m = ctx.start_loop();
                    let fragment = Fragment::with_compile(effects, ctx);
                    let len = fragment.len() as isize;
                    m.finish(ctx);
                    (fragment, len)
                };
                let mut fragment = Fragment::new();
                fragment
                    .append_fragment(cond_fragment)
                    .append(JumpIfFalse(effects_len + 2))
                    .append_fragment(effects_fragment)
                    .append(Jump(-(effects_len + 1 + cond_len)));
                fragment.patch_forward_jump(1);
                fragment.patch_backward_jump(0);
                fragment
            };
            fragment.append_fragment(while_fragment);
        }

        Effect::Scope { body } => {
            let m = ctx.start_block();
            fragment
                .append_compile(body, ctx)
                .append(DropLocal(ctx.get_block_local_count()));
            m.finish(ctx);
        }

        Effect::Call { value, args } => {
            fragment.append_compile(value, ctx);
            assert!(
                args.len() <= u8::MAX as usize,
                "Number of arguments greater than u8::MAX is not supported."
            );
            let mut args_range = Vec::with_capacity(args.len());
            for (syntax, arg) in ctx.strage.get(args) {
                fragment.append_compile(&arg, ctx);
                args_range.push(syntax.text_range());
            }
            fragment
                .append(Call(args.len() as u8, args_range.into_boxed_slice()))
                .append(Unload);
        }

        Effect::MethodCall { table, name, args } => {
            fragment.append_compile(table, ctx);
            let name_string = ctx.strage.get(name).unwrap().1.clone();
            assert!(
                args.len() <= u8::MAX as usize,
                "Number of arguments greater than u8::MAX is not supported."
            );
            let mut args_range = Vec::with_capacity(args.len());
            for (syntax, arg) in ctx.strage.get(args) {
                fragment.append_compile(&arg, ctx);
                args_range.push(syntax.text_range());
            }
            fragment
                .append(CallMethod(
                    args.len() as u8,
                    UString::from(name_string),
                    args_range.into_boxed_slice(),
                ))
                .append(Unload);
        }

        Effect::Return { value } => {
            match ctx.strage.get(value) {
                Some((_, value)) => fragment.append_compile(&value, ctx),
                None => fragment.append(LoadNilObject),
            };
            fragment.append(Leave);
        }

        Effect::BreakLoop => {
            fragment
                .append(DropLocal(ctx.get_loop_local_count()))
                .append_forward_jump();
        }

        Effect::ContinueLoop => {
            fragment
                .append(DropLocal(ctx.get_loop_local_count()))
                .append_backward_jump();
        }

        Effect::NoEffectValue { value } => {
            fragment.append_compile(value, ctx).append(Unload);
        }
    }
}
