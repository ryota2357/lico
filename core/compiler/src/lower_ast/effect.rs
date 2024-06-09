use super::*;
use core::iter;

pub(super) fn effect(ctx: &mut Context, statement: ast::Statement) -> ir::Effect {
    match statement {
        // var [name] = [initializer]
        ast::Statement::Var(node) => {
            let symbol = node.name().and_then(|n| n.ident_token()).map(|token| {
                let scope = ctx.scope_index();
                let text = CompactString::from(token.text());
                (token, ir::Symbol::new(text, scope))
            });
            let value = node.initializer().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            ir::Effect::MakeLocal {
                name: ctx.builder.add_symbol(symbol),
                value: ctx.builder.add_value(value),
            }
        }

        // func [name_path] ('->' [name])? [params]
        //     [body]
        // end
        ast::Statement::Func(node) => {
            let func_key = {
                let scope = ctx.start_scope(ScopeKind::New);
                let params = node
                    .param_list()
                    .map(|params| params.into_lowered(ctx))
                    .unwrap_or_default();
                let body: Vec<_> = node
                    .body()
                    .map(|body| body.into_lowered(ctx))
                    .unwrap_or_default();
                scope.finish(ctx);
                ctx.builder.add_function(params, body)
            };
            fn to_text_token(name: ast::Name) -> Option<(SyntaxToken, CompactString)> {
                let token = name.ident_token()?;
                let text = CompactString::new(token.text());
                Some((token, text))
            }
            let (root_symbol, path) = {
                let mut name_paths = {
                    let mut paths = Vec::new();
                    let mut name_path = node.name_path();
                    while let Some(path) = name_path {
                        let name = path.name();
                        let sn = path.syntax().clone();
                        paths.push((sn, name));
                        name_path = path.child();
                    }
                    paths
                };
                let root_symbol = name_paths.pop().and_then(|(_, name)| {
                    let (token, text) = to_text_token(name?)?;
                    let symbol = ir::Symbol::new(text, ctx.scope_index());
                    Some((token, symbol))
                });
                let path = name_paths.into_iter().map(|(_, name)| {
                    let token = match name.and_then(|name| name.ident_token()) {
                        Some(token) => token,
                        None => todo!("handle error?"),
                    };
                    let text = CompactString::new(token.text());
                    (token, text)
                });
                (root_symbol, path)
            };
            if let Some(method_name) = node.method_name() {
                ir::Effect::SetMethod {
                    table: ctx.builder.add_symbol(root_symbol),
                    path: ctx.builder.add_string_many(path),
                    name: ctx.builder.add_string(to_text_token(method_name)),
                    func: func_key,
                }
            } else {
                match path.len() {
                    0 => ir::Effect::MakeFunc {
                        name: ctx.builder.add_symbol(root_symbol),
                        func: func_key,
                    },
                    _ => ir::Effect::SetFieldFunc {
                        table: ctx.builder.add_symbol(root_symbol),
                        path: ctx.builder.add_string_many(path),
                        func: func_key,
                    },
                }
            }
        }

        // for [name] in [iterable] do [body] end
        ast::Statement::For(node) => {
            let iterable = node.iterable().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let scope = ctx.start_scope(ScopeKind::Loop);
            let variable = node.name().and_then(|n| n.ident_token()).map(|token| {
                let scope = ctx.scope_index();
                let text = CompactString::from(token.text());
                (token, ir::Symbol::new(text, scope))
            });
            let effects = match node.body() {
                Some(body) => body.into_lowered(ctx),
                None => Vec::new(),
            };
            scope.finish(ctx);
            ir::Effect::LoopFor {
                variable: ctx.builder.add_symbol(variable),
                iterable: ctx.builder.add_value(iterable),
                effects: ctx.builder.add_effects(effects),
            }
        }

        // while [cond] do [body] end
        ast::Statement::While(node) => {
            let condition = node.condition().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let scope = ctx.start_scope(ScopeKind::Loop);
            let effects = match node.body() {
                Some(body) => body.into_lowered(ctx),
                None => Vec::new(),
            };
            scope.finish(ctx);
            ir::Effect::LoopWhile {
                condition: ctx.builder.add_value(condition),
                effects: ctx.builder.add_effects(effects),
            }
        }

        // return [expr]
        ast::Statement::Return(node) => {
            let value = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            ir::Effect::Return {
                value: ctx.builder.add_value(value),
            }
        }

        // break
        ast::Statement::Break(node) => {
            if !ctx.is_in_loop() {
                let range = node.syntax().text_range();
                ctx.push_error("'break' outside of loop", range);
            }
            ir::Effect::BreakLoop
        }

        // continue
        ast::Statement::Continue(node) => {
            if !ctx.is_in_loop() {
                let range = node.syntax().text_range();
                ctx.push_error("'continue' outside of loop", range);
            }
            ir::Effect::ContinueLoop
        }

        ast::Statement::Expr(node) => {
            let Some(mut expr) = node.expr() else {
                return ir::Effect::Scope {
                    body: ctx.builder.add_effects([]),
                };
            };
            while let ast::Expression::Paren(paren) = expr {
                let Some(inner) = paren.expr() else {
                    return ir::Effect::Scope {
                        body: ctx.builder.add_effects([]),
                    };
                };
                expr = inner;
            }
            let no_effect_value = match expr {
                ast::Expression::If(node) => return if_(ctx, node),
                ast::Expression::Do(node) => return do_(ctx, node),
                ast::Expression::Call(node) => return call_(ctx, node),
                ast::Expression::Binary(node) => return binary_(ctx, node),
                ast::Expression::MethodCall(node) => return method_call_(ctx, node),
                ast::Expression::Paren(_) => {
                    unreachable!("Paren expression is unwrapped above while loop");
                }
                ast::Expression::Prefix(node) => {
                    let op = node.op().map(|(_, op)| op);
                    let range = node.syntax().text_range();
                    let message = format!(
                        "{} expression result is not used",
                        match op {
                            Some(op) => format!("`{}`", op.sign_text()),
                            None => "Prefix".to_string(),
                        }
                    );
                    ctx.push_error(message, range);
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::Prefix(node)))
                }
                ast::Expression::Index(node) => {
                    let range = node.syntax().text_range();
                    ctx.push_error("Index access result is not used", range);
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::Index(node)))
                }
                ast::Expression::Field(node) => {
                    let range = node.syntax().text_range();
                    ctx.push_error("Field access result is not used", range);
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::Field(node)))
                }
                ast::Expression::LocalVar(node) => {
                    ctx.push_error("Unexpected local variable", node.syntax().text_range());
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::LocalVar(node)))
                }
                ast::Expression::Literal(node) => {
                    ctx.push_error("Unexpected literal value", node.syntax().text_range());
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::Literal(node)))
                }
                ast::Expression::ArrayConst(node) => {
                    ctx.push_error("Unexpected array object", node.syntax().text_range());
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::ArrayConst(node)))
                }
                ast::Expression::TableConst(node) => {
                    ctx.push_error("Unexpected table object", node.syntax().text_range());
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::TableConst(node)))
                }
                ast::Expression::FuncConst(node) => {
                    ctx.push_error("Unexpected function object", node.syntax().text_range());
                    let sn = node.syntax().clone();
                    (sn, value(ctx, ast::Expression::FuncConst(node)))
                }
            };
            ir::Effect::NoEffectValue {
                value: ctx.builder.add_value(no_effect_value),
            }
        }

        ast::Statement::Attr(_) => {
            unimplemented!("Attribute statement");
        }
    }
}

fn do_(ctx: &mut Context, node: ast::DoExpr) -> ir::Effect {
    let scope = ctx.start_scope(ScopeKind::Nest);
    let effects: Vec<_> = node
        .body()
        .map(|body| body.into_lowered(ctx))
        .unwrap_or_default();
    scope.finish(ctx);
    ir::Effect::Scope {
        body: ctx.builder.add_effects(effects),
    }
}

fn call_(ctx: &mut Context, node: ast::CallExpr) -> ir::Effect {
    let value = node.expr().map(|expr| {
        let sn = expr.syntax().clone();
        (sn, value(ctx, expr))
    });
    let args = node
        .arg_list()
        .map(|args| args.into_lowered(ctx))
        .unwrap_or_default();
    ir::Effect::Call {
        value: ctx.builder.add_value(value),
        args: ctx.builder.add_value_many(args),
    }
}

fn method_call_(ctx: &mut Context, node: ast::MethodCallExpr) -> ir::Effect {
    let value = node.expr().map(|expr| {
        let sn = expr.syntax().clone();
        (sn, value(ctx, expr))
    });
    let name = node
        .method_name()
        .and_then(|name| name.ident_token())
        .map(|token| {
            let text = CompactString::from(token.text());
            (token, text)
        });
    let args = node
        .arg_list()
        .map(|args| args.into_lowered(ctx))
        .unwrap_or_default();
    ir::Effect::MethodCall {
        table: ctx.builder.add_value(value),
        name: ctx.builder.add_string(name),
        args: ctx.builder.add_value_many(args),
    }
}

fn if_(ctx: &mut Context, node: ast::IfExpr) -> ir::Effect {
    let condition = node.condition().map(|expr| {
        let sn = expr.syntax().clone();
        (sn, value(ctx, expr))
    });

    let scope = ctx.start_scope(ScopeKind::Nest);
    let then: Vec<(_, ir::Effect)> = node
        .body()
        .map(|body| body.into_lowered(ctx))
        .unwrap_or_default();
    scope.finish(ctx);

    let (has_elif, elif_branches) = {
        let iter = node.elif_branches().map(|elif_branch| {
            let condition = elif_branch.condition().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let scope = ctx.start_scope(ScopeKind::Nest);
            let body: Vec<(_, ir::Effect)> = elif_branch
                .body()
                .map(|body| body.into_lowered(ctx))
                .unwrap_or_default();
            scope.finish(ctx);
            let sn = elif_branch.syntax().clone();
            (sn, condition, body)
        });
        // To make `iter` double ended iterator, we need to collect it to Vec.
        // (AstChildren is not exact size iterator and is not double ended iterator.)
        let vec = iter.collect::<Vec<_>>();
        (!vec.is_empty(), vec.into_iter())
    };

    let scope = ctx.start_scope(ScopeKind::Nest);
    let else_: Vec<(_, ir::Effect)> = node
        .else_branch()
        .and_then(|else_branch| else_branch.body().map(|body| body.into_lowered(ctx)))
        .unwrap_or_default();
    scope.finish(ctx);

    if has_elif {
        // if .. then .. [elif .. then ..]+ end
        // if .. then .. [elif .. then ..]+ else .. end
        let else_ = {
            let mut elif_branches = elif_branches.rev();
            let tail_branch = {
                // `elif_branches` should not be empty since `has_elif` is true
                let (syntax_node, condition, body) = elif_branches.next().unwrap();
                let branch = ir::Effect::Branch {
                    condition: ctx.builder.add_value(condition),
                    then: ctx.builder.add_effects(body),
                    else_: ctx.builder.add_effects(else_),
                };
                (syntax_node, branch)
            };
            elif_branches.fold(tail_branch, |branch, above_elif| {
                let (syntax_node, condition, body) = above_elif;
                let branch = ir::Effect::Branch {
                    condition: ctx.builder.add_value(condition),
                    then: ctx.builder.add_effects(body),
                    else_: ctx.builder.add_effects(iter::once(branch)),
                };
                (syntax_node, branch)
            })
        };
        ir::Effect::Branch {
            condition: ctx.builder.add_value(condition),
            then: ctx.builder.add_effects(then),
            else_: ctx.builder.add_effects(iter::once(else_)),
        }
    } else {
        ir::Effect::Branch {
            condition: ctx.builder.add_value(condition),
            then: ctx.builder.add_effects(then),
            else_: ctx.builder.add_effects(else_),
        }
    }
}

fn binary_(ctx: &mut Context, node: ast::BinaryExpr) -> ir::Effect {
    let op_token = match node.op() {
        Some((token, ast::BinaryOp::Assign)) => token,
        op => {
            let op = op.map(|(_, op)| op);
            let range = node.syntax().text_range();
            let message = format!(
                "{} expression result is not used",
                match op {
                    Some(op) => format!("`{}`", op.sign_text()),
                    None => "Binary".to_string(),
                }
            );
            ctx.push_error(message, range);
            let value = {
                let sn = node.syntax().clone();
                (sn, value(ctx, ast::Expression::Binary(node)))
            };
            return ir::Effect::NoEffectValue {
                value: ctx.builder.add_value(value),
            };
        }
    };

    let rhs = node.rhs().map(|expr| {
        let sn = expr.syntax().clone();
        (sn, value(ctx, expr))
    });

    let lhs = {
        fn unwrap_paren(mut expr: ast::Expression) -> Option<ast::Expression> {
            while let ast::Expression::Paren(paren) = expr {
                expr = paren.expr()?;
            }
            Some(expr)
        }
        match node.lhs().and_then(unwrap_paren) {
            Some(lhs) => lhs,
            None => {
                ctx.push_error("Invalid left-hand side expression", op_token.text_range());
                // lhs error is no need to report an error because `()` expression (invalid) should be
                // reported by the parser.
                return ir::Effect::SetLocal {
                    local: ctx.builder.add_symbol(None),
                    value: ctx.builder.add_value(rhs),
                };
            }
        }
    };
    match lhs {
        ast::Expression::Index(node) => {
            let target = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let index = node.index().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            ir::Effect::SetIndex {
                target: ctx.builder.add_value(target),
                index: ctx.builder.add_value(index),
                value: ctx.builder.add_value(rhs),
            }
        }
        ast::Expression::Field(node) => {
            let target = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let field = node
                .field()
                .and_then(|name| name.ident_token())
                .map(|token| {
                    let string = CompactString::from(token.text());
                    (token, string)
                });
            ir::Effect::SetField {
                target: ctx.builder.add_value(target),
                field: ctx.builder.add_string(field),
                value: ctx.builder.add_value(rhs),
            }
        }
        ast::Expression::LocalVar(node) => {
            let symbol = node.ident_token().map(|token| {
                let scope = ctx.scope_index();
                let text = CompactString::from(token.text());
                (token, ir::Symbol::new(text, scope))
            });
            ir::Effect::SetLocal {
                local: ctx.builder.add_symbol(symbol),
                value: ctx.builder.add_value(rhs),
            }
        }
        ast::Expression::Paren(_) => unreachable!("Paren expression is unwrapped above"),
        ast::Expression::If(_) => todo!(),
        ast::Expression::Do(_) => todo!(),
        ast::Expression::Call(_) => todo!(),
        ast::Expression::MethodCall(_) => todo!(),
        ast::Expression::Binary(_) => todo!(),
        ast::Expression::Prefix(_) => todo!(),
        ast::Expression::Literal(_) => todo!(),
        ast::Expression::ArrayConst(_) => todo!(),
        ast::Expression::TableConst(_) => todo!(),
        ast::Expression::FuncConst(_) => todo!(),
    }
}
