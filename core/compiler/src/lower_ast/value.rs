use super::*;
use compact_str::CompactString;
use foundation::object::UString;

pub(super) fn value(ctx: &mut Context, expression: ast::Expression) -> ir::Value {
    match expression {
        ast::Expression::If(node) => {
            let condition = node.condition().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let (then, then_tail) = match node.body() {
                Some(then_branch) => then_branch.into_lowered(ctx),
                None => {
                    let range = node.syntax().text_range();
                    ctx.push_error("'if' expression requires body after 'then'", range);
                    (Vec::new(), None)
                }
            };
            let (has_elif, elif_branches) = {
                let iter = node.elif_branches().map(|elif_branch| {
                    let condition = elif_branch.condition().map(|expr| {
                        let sn = expr.syntax().clone();
                        (sn, value(ctx, expr))
                    });
                    let (body, tail) = match elif_branch.body() {
                        Some(body) => body.into_lowered(ctx),
                        None => {
                            let range = elif_branch.syntax().text_range();
                            ctx.push_error("'elif' branch requires body", range);
                            (Vec::new(), None)
                        }
                    };
                    let sn = elif_branch.syntax().clone();
                    (sn, condition, body, tail)
                });
                // NOTE: AstChildren is not exact size iterator and is not double ended iterator.
                let vec = iter.collect::<Vec<_>>();
                (!vec.is_empty(), vec.into_iter())
            };
            let (has_else, else_, else_tail) = match node.else_branch() {
                Some(else_branch) => {
                    let (else_, else_tail) = match else_branch.body() {
                        Some(body) => body.into_lowered(ctx),
                        None => {
                            let range = else_branch.syntax().text_range();
                            ctx.push_error("'else' branch requires body", range);
                            (Vec::new(), None)
                        }
                    };
                    (true, else_, else_tail)
                }
                None => (false, Vec::new(), None),
            };

            if has_elif {
                // if .. then .. [elif .. then ..]+ end
                // if .. then .. [elif .. then ..]+ else .. end
                let mut elif_branches = elif_branches.rev();
                let tail_branch = {
                    // `elif_branches` should not be empty since `has_elif` is true"
                    let (syntax_node, condition, body, tail) = elif_branches.next().unwrap();
                    if !has_else {
                        let range = syntax_node.text_range();
                        ctx.push_error("'if' expression requires 'else' branch", range);
                    }
                    let branch = ir::Value::Branch {
                        condition: ctx.builder.add_value(condition),
                        then: ctx.builder.add_effects(body),
                        then_tail: ctx.builder.add_value(tail),
                        else_: ctx.builder.add_effects(else_),
                        else_tail: ctx.builder.add_value(else_tail),
                    };
                    (syntax_node, branch)
                };
                let else_tail = elif_branches.fold(tail_branch, |branch, above_elif| {
                    let (syntax_node, condition, body, tail) = above_elif;
                    let branch = ir::Value::Branch {
                        condition: ctx.builder.add_value(condition),
                        then: ctx.builder.add_effects(body),
                        then_tail: ctx.builder.add_value(tail),
                        else_: ctx.builder.add_effects([]),
                        else_tail: ctx.builder.add_value(branch),
                    };
                    (syntax_node, branch)
                });
                ir::Value::Branch {
                    condition: ctx.builder.add_value(condition),
                    then: ctx.builder.add_effects(then),
                    then_tail: ctx.builder.add_value(then_tail),
                    else_: ctx.builder.add_effects([]),
                    else_tail: ctx.builder.add_value(else_tail),
                }
            } else {
                // if .. then .. end
                // if .. then .. else .. end
                if !has_else {
                    let range = node.syntax().text_range();
                    ctx.push_error("'if' expression requires 'else' branch", range);
                }
                ir::Value::Branch {
                    condition: ctx.builder.add_value(condition),
                    then: ctx.builder.add_effects(then),
                    then_tail: ctx.builder.add_value(then_tail),
                    else_: ctx.builder.add_effects(else_),
                    else_tail: ctx.builder.add_value(else_tail),
                }
            }
        }

        // do [body] end
        ast::Expression::Do(node) => {
            let scope = ctx.start_scope(ScopeKind::Nest);
            let (effects, tail) = match node.body() {
                Some(body) => body.into_lowered(ctx),
                None => {
                    ctx.push_error("'do' block requires body", node.syntax().text_range());
                    return ir::Value::Block {
                        effects: ctx.builder.add_effects([]),
                        tail: ctx.builder.add_value(None),
                    };
                }
            };
            scope.finish(ctx);
            ir::Value::Block {
                effects: ctx.builder.add_effects(effects),
                tail: ctx.builder.add_value(tail),
            }
        }

        // [expr]([arg_list])
        ast::Expression::Call(node) => {
            let value = node
                .expr()
                .map(|expr| ((expr.syntax().clone()), value(ctx, expr)));
            let args = node.arg_list().map(|arg_list| arg_list.into_lowered(ctx));
            ir::Value::Call {
                value: ctx.builder.add_value(value),
                args: ctx.builder.add_value_many(args.unwrap_or(Vec::new())),
            }
        }

        // [lhs] [op] [rhs]
        ast::Expression::Binary(node) => {
            let lhs = node.lhs().map(|lhs| {
                let sn = lhs.syntax().clone();
                (sn, value(ctx, lhs))
            });
            let rhs = node.rhs().map(|rhs| {
                let sn = rhs.syntax().clone();
                (sn, value(ctx, rhs))
            });
            ir::Value::Binary {
                lhs: ctx.builder.add_value(lhs),
                rhs: ctx.builder.add_value(rhs),
                op: match node.op() {
                    Some((token, op)) => match op {
                        ast::BinaryOp::Add => ir::BinaryOp::Add(token),
                        ast::BinaryOp::Sub => ir::BinaryOp::Sub(token),
                        ast::BinaryOp::Mul => ir::BinaryOp::Mul(token),
                        ast::BinaryOp::Div => ir::BinaryOp::Div(token),
                        ast::BinaryOp::Mod => ir::BinaryOp::Mod(token),
                        ast::BinaryOp::Shl => ir::BinaryOp::Shl(token),
                        ast::BinaryOp::Shr => ir::BinaryOp::Shr(token),
                        ast::BinaryOp::Concat => ir::BinaryOp::Concat(token),
                        ast::BinaryOp::Eq => ir::BinaryOp::Eq(token),
                        ast::BinaryOp::Ne => ir::BinaryOp::Ne(token),
                        ast::BinaryOp::Lt => ir::BinaryOp::Lt(token),
                        ast::BinaryOp::Le => ir::BinaryOp::Le(token),
                        ast::BinaryOp::Gt => ir::BinaryOp::Gt(token),
                        ast::BinaryOp::Ge => ir::BinaryOp::Ge(token),
                        ast::BinaryOp::And => ir::BinaryOp::And(token),
                        ast::BinaryOp::Or => ir::BinaryOp::Or(token),
                        ast::BinaryOp::BitAnd => ir::BinaryOp::BitAnd(token),
                        ast::BinaryOp::BitOr => ir::BinaryOp::BitOr(token),
                        ast::BinaryOp::BitXor => ir::BinaryOp::BitXor(token),
                        ast::BinaryOp::Assign => {
                            ctx.push_error(
                                "Assignment operator is not allowed in expression. For equality comparison, use `==` instead of `=`.",
                                token.text_range(),
                            );
                            ir::BinaryOp::Assign(token)
                        }
                    },
                    None => ir::BinaryOp::Missing,
                },
            }
        }

        // [op] [expr]
        ast::Expression::Prefix(node) => {
            let value = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            ir::Value::Prefix {
                value: ctx.builder.add_value(value),
                op: match node.op() {
                    Some((token, op)) => match op {
                        ast::PrefixOp::Plus => ir::PrefixOp::Plus(token),
                        ast::PrefixOp::Minus => ir::PrefixOp::Minus(token),
                        ast::PrefixOp::Not => ir::PrefixOp::Not(token),
                        ast::PrefixOp::BitNot => ir::PrefixOp::BitNot(token),
                    },
                    None => ir::PrefixOp::Missing,
                },
            }
        }

        // [expr] [ [expr] ]
        ast::Expression::Index(node) => {
            let value_ = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let index = node.index().map(|index| {
                let sn = index.syntax().clone();
                (sn, value(ctx, index))
            });
            ir::Value::Index {
                value: ctx.builder.add_value(value_),
                index: ctx.builder.add_value(index),
            }
        }

        // [expr].[ident]
        ast::Expression::Field(node) => {
            let value = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let name = node
                .field()
                .and_then(|field| field.ident_token())
                .map(|token| {
                    let string = CompactString::from(token.text());
                    (token, string)
                });
            ir::Value::Field {
                value: ctx.builder.add_value(value),
                name: ctx.builder.add_string(name),
            }
        }

        // [expr]->[ident]( [arg_list] )
        ast::Expression::MethodCall(node) => {
            let value = node.expr().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let name = node
                .method_name()
                .and_then(|name| name.ident_token())
                .map(|token| {
                    let string = CompactString::from(token.text());
                    (token, string)
                });
            let args = match node.arg_list() {
                Some(arg_list) => arg_list.into_lowered(ctx),
                None => Vec::new(),
            };
            ir::Value::MethodCall {
                value: ctx.builder.add_value(value),
                name: ctx.builder.add_string(name),
                args: ctx.builder.add_value_many(args),
            }
        }

        // ([expr])
        ast::Expression::Paren(node) => match node.expr() {
            Some(expr) => value(ctx, expr),
            None => ir::Value::Nil, // `()` fallback to `nil` (error is reported in parser)
        },

        // [ident]
        ast::Expression::LocalVar(node) => {
            let symbol = node.ident_token().map(|token| {
                let scope = ctx.scope_index();
                let text = CompactString::from(token.text());
                (token, ir::Symbol::new(text, scope))
            });
            ir::Value::Local {
                name: ctx.builder.add_symbol(symbol),
            }
        }

        ast::Expression::Literal(node) => {
            let Some(literal_kind) = node.kind() else {
                panic!("LiteralKind is missing, this should be handled in parser");
            };
            fn remove_underscore(text: &str) -> CompactString {
                let chars = text.chars();
                let mut text = CompactString::new("");
                for c in chars {
                    if c == '_' {
                        continue;
                    }
                    text.push(c);
                }
                text
            }
            match literal_kind {
                ast::LiteralKind::Int(token) => {
                    let mut text = token.text();
                    let base = if text.starts_with("0b") {
                        text = &text[2..];
                        2
                    } else if text.starts_with("0o") {
                        text = &text[2..];
                        8
                    } else if text.starts_with("0x") {
                        text = &text[2..];
                        16
                    } else {
                        10
                    };
                    let value = match i64::from_str_radix(&remove_underscore(text), base) {
                        Ok(x) => x,
                        Err(e) => {
                            use core::num::IntErrorKind;
                            match e.kind() {
                                IntErrorKind::Empty => {} // this error is handled in parser
                                IntErrorKind::InvalidDigit => {
                                    ctx.push_error("Contains invalid digit", token.text_range())
                                }
                                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                                    ctx.push_error("64-bit integer overflow", token.text_range());
                                }
                                IntErrorKind::Zero => unreachable!("i64 is not non-zero"),
                                &_ => unreachable!("Why?, what is match here?"),
                            }
                            0
                        }
                    };
                    ir::Value::Int(value)
                }
                ast::LiteralKind::Float(token) => {
                    let text = token.text();
                    let value = match remove_underscore(text).parse::<f64>() {
                        Ok(x) => x,
                        Err(e) => {
                            ctx.push_error(format!("Invalid float: {}", e), token.text_range());
                            0.0
                        }
                    };
                    ir::Value::Float(value)
                }
                ast::LiteralKind::String(token) => {
                    let mut text = token.text();
                    let start = text.chars().next().unwrap_or('\0');
                    let end = text.chars().last().unwrap_or('\0');
                    debug_assert!(
                        ['"', '\''].contains(&start),
                        "We expected all strings must start with `\"` or `'`, because these char are used in lexer as marker of string start. but got: {:?}",
                        start
                    );
                    if start == end {
                        text = &text[1..(text.len() - 1)];
                    } else {
                        // non-terminated error is handled in parser
                        text = &text[1..text.len()];
                    }
                    ir::Value::String(UString::from(text))
                }
                ast::LiteralKind::Bool(value) => ir::Value::Bool(value),
                ast::LiteralKind::Nil => ir::Value::Nil,
            }
        }

        // [ [expr], [expr], ... ]
        ast::Expression::ArrayConst(node) => {
            let elements = node.elements().map(|expr| {
                let sn = expr.syntax().clone();
                (sn, value(ctx, expr))
            });
            let elements_vec = elements.collect::<Vec<_>>(); // avoid `ctx` double mutable borrow
            ir::Value::Array {
                elements: ctx.builder.add_value_many(elements_vec),
            }
        }

        // { [field], [field], ... }
        ast::Expression::TableConst(node) => {
            let fields = {
                let mut fields = Vec::new();
                for field in node.fields() {
                    let key_name = match field.field_name() {
                        Some(ast::TableFieldName::Expr(expr)) => match expr.expr() {
                            Some(expr) => {
                                let sn = expr.syntax().clone();
                                let value = value(ctx, expr);
                                let value_key = ctx.builder.add_value((sn, value));
                                ir::TableKeyName::Value(value_key)
                            }
                            None => {
                                let string_key = ctx.builder.add_string(None);
                                ir::TableKeyName::String(string_key)
                            }
                        },
                        Some(ast::TableFieldName::Ident(ident)) => {
                            let string_key = match ident.ident_token() {
                                Some(ident) => {
                                    let string = CompactString::from(ident.text());
                                    ctx.builder.add_string((ident, string))
                                }
                                None => ctx.builder.add_string(None),
                            };
                            ir::TableKeyName::String(string_key)
                        }
                        None => {
                            let string_key = ctx.builder.add_string(None);
                            ir::TableKeyName::String(string_key)
                        }
                    };
                    let initializer = match field.initializer() {
                        Some(expr) => {
                            let sn = expr.syntax().clone();
                            let value = value(ctx, expr);
                            ctx.builder.add_value((sn, value))
                        }
                        None => ctx.builder.add_value(None),
                    };
                    fields.push((key_name, initializer));
                }
                fields.into_boxed_slice()
            };
            ir::Value::Table { fields }
        }

        // func [param_list] [body] end
        ast::Expression::FuncConst(node) => {
            let scope = ctx.start_scope(ScopeKind::New);
            let params = match node.param_list() {
                Some(param_list) => param_list.into_lowered(ctx),
                None => Vec::new(),
            };
            let (mut effects, value) = match node.body() {
                Some(body) => body.into_lowered(ctx),
                None => (Vec::new(), None),
            };
            if let Some(value) = value {
                let syntax_node = value.0.clone();
                effects.push((
                    syntax_node,
                    ir::Effect::Return {
                        value: ctx.builder.add_value(value),
                    },
                ));
            }
            scope.finish(ctx);
            let func_key = ctx.builder.add_function(params, effects);
            ir::Value::Function(func_key)
        }
    }
}
