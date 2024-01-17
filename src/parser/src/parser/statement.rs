use super::*;

impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
    /// None = (Token::Error | Token::Comment)* EOF
    pub fn statement(&mut self) -> Option<(Statement<'src>, TextSpan)> {
        let Some((token, span)) = self.next() else {
            return None;
        };
        self.statement_with(token, span)
    }

    /// None = (Token::Error | Token::Comment)* EOF
    pub fn statement_with(
        &mut self,
        token: &'tokens Token<'src>,
        span: TextSpan,
    ) -> Option<(Statement<'src>, TextSpan)> {
        match token {
            Token::Int(_) => todo!(),
            Token::Float(_) => todo!(),
            Token::String(_) => todo!(),
            Token::Bool(_) => todo!(),
            Token::Nil => todo!(),

            // keywords
            Token::Var => Some(self.var_statement(span)),
            Token::Func => Some(self.func_statement(span)),
            Token::If => Some(self.if_statement(span)),
            Token::Then => todo!(),
            Token::Elif => todo!(),
            Token::Else => todo!(),
            Token::For => Some(self.for_statement(span)),
            Token::While => Some(self.while_statement(span)),
            Token::In => todo!(),
            Token::Ref => todo!(),
            Token::Do => Some(self.do_statement(span)),
            Token::End => todo!(),
            Token::Return => Some(self.return_statement(span)),
            Token::Break => Some((Statement::Break, span)),
            Token::Continue => Some((Statement::Continue, span)),

            // operators
            Token::Plus => {
                self.report(Error::UnexpectedSymbol("+", span));
                Some((Statement::Error, span))
            }
            Token::Minus => {
                self.report(Error::UnexpectedSymbol("-", span));
                Some((Statement::Error, span))
            }
            Token::Star => {
                self.report(Error::UnexpectedSymbol("*", span));
                Some((Statement::Error, span))
            }
            Token::Div => {
                self.report(Error::UnexpectedSymbol("/", span));
                Some((Statement::Error, span))
            }
            Token::Mod => {
                self.report(Error::UnexpectedSymbol("%", span));
                Some((Statement::Error, span))
            }
            Token::Eq => {
                self.report(Error::UnexpectedSymbol("=", span));
                Some((Statement::Error, span))
            }
            Token::NotEq => {
                self.report(Error::UnexpectedSymbol("!=", span));
                Some((Statement::Error, span))
            }
            Token::Less => {
                self.report(Error::UnexpectedSymbol("<", span));
                Some((Statement::Error, span))
            }
            Token::LessEq => {
                self.report(Error::UnexpectedSymbol("<=", span));
                Some((Statement::Error, span))
            }
            Token::Greater => {
                self.report(Error::UnexpectedSymbol(">", span));
                Some((Statement::Error, span))
            }
            Token::GreaterEq => {
                self.report(Error::UnexpectedSymbol(">=", span));
                Some((Statement::Error, span))
            }
            Token::Dot => {
                self.report(Error::UnexpectedSymbol(".", span));
                Some((Statement::Error, span))
            }
            Token::Arrow => {
                self.report(Error::UnexpectedSymbol("->", span));
                Some((Statement::Error, span))
            }
            Token::Dot2 => {
                self.report(Error::UnexpectedSymbol("..", span));
                Some((Statement::Error, span))
            }
            Token::Assign => {
                self.report(Error::UnexpectedSymbol("=", span));
                Some((Statement::Error, span))
            }

            // keyword operators
            Token::And => {
                self.report(Error::UnexpectedSymbol("and", span));
                Some((Statement::Error, span))
            }
            Token::Or => {
                self.report(Error::UnexpectedSymbol("or", span));
                Some((Statement::Error, span))
            }
            Token::Not => {
                self.report(Error::UnexpectedSymbol("not", span));
                Some((Statement::Error, span))
            }

            // delimiters
            Token::Comma => {
                self.report(Error::UnexpectedSymbol(",", span));
                Some((Statement::Error, span))
            }
            Token::Colon => {
                self.report(Error::UnexpectedSymbol(":", span));
                Some((Statement::Error, span))
            }
            Token::OpenParen => {
                self.move_prev();
                // SAFETY: When next token is OpenParen '(', the next element is delimited expression.
                //         So `self.expression()` returns Some.
                let (expr, expr_span) = unsafe { self.expression().unwrap_unchecked() };
                Some(self.expr_to_statement(expr, expr_span))
            }
            Token::CloseParen => {
                self.report(Error::UnexpectedSymbol(")", span));
                Some((Statement::Error, span))
            }
            Token::OpenBrace => {
                self.move_prev();
                // SAFETY: When next token is OpenBrace '{', the next element is table-obj expression.
                //        So `self.expression()` returns Some.
                let (expr, expr_span) = unsafe { self.expression().unwrap_unchecked() };
                Some(self.expr_to_statement(expr, expr_span))
            }
            Token::CloseBrace => {
                self.report(Error::UnexpectedSymbol("}", span));
                Some((Statement::Error, span))
            }
            Token::OpenBracket => {
                self.move_prev();
                // SAFETY: When next token is OpenBracket '[', the next element is array-obj expression.
                //        So `self.expression()` returns Some.
                let (expr, expr_span) = unsafe { self.expression().unwrap_unchecked() };
                Some(self.expr_to_statement(expr, expr_span))
            }
            Token::CloseBracket => {
                self.report(Error::UnexpectedSymbol("]", span));
                Some((Statement::Error, span))
            }

            // other
            Token::Ident(name) => Some(self.assign_or_call_statement(name, span)),
            Token::Attribute(name) => Some(self.attribute_statement(name, span)),
            Token::Comment(_) | Token::Error(_) => {
                // skip
                self.statement()
            }
        }
    }

    // var [name] = [expr]
    fn var_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        let (name, name_span) = match self.next() {
            Some((Token::Ident(name), span)) => (*name, span),
            Some((Token::Assign, assign_span)) => {
                let span = TextSpan::new(start_span.end(), assign_span.start());
                self.report(Error::MissingRequiredElement("<name>", span));
                ("$dummy", span)
            }
            Some((token, span)) => {
                self.report(Error::UnexpectedSymbol("var", start_span));
                return self.statement_with(token, span).unwrap_or_else(|| {
                    let span = TextSpan::new(start_span.start(), span.end());
                    (Statement::Error, span)
                });
            }
            None => {
                let span = TextSpan::new(start_span.start(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("<name>", span));
                return (Statement::Error, span);
            }
        };
        if let Some((Token::Assign, _)) = self.look(0) {
            self.move_next();
        } else {
            let span = TextSpan::new(start_span.start(), name_span.end());
            self.report(Error::MissingRequiredElement("= <expr>", span));
            return (
                Statement::Var {
                    name: (name, name_span),
                    expr: (Expression::Error, TextSpan::at(name_span.end(), 0)),
                },
                span,
            );
        }
        let (expr, expr_span) = match self.expression() {
            Some((expr, span)) => (expr, span),
            None => {
                let span = TextSpan::new(start_span.start(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("<expr>", span));
                return (Statement::Error, span);
            }
        };
        (
            Statement::Var {
                name: (name, name_span),
                expr: (expr, expr_span),
            },
            TextSpan::new(start_span.start(), expr_span.end()),
        )
    }

    // func [name]([args])
    //     [block]
    // end
    fn func_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        let (name, fields, args) = match self.next() {
            Some((Token::Ident(name), name_span)) => {
                let mut fields = Vec::new();
                let (name, args) = loop {
                    match self.look(0) {
                        Some((Token::OpenParen, _)) => {
                            self.move_next();
                            let (args, _) = self.func_def_args();
                            break ((*name, name_span), args);
                        }
                        Some((Token::Dot, _)) => {
                            self.move_next();
                            match self.look(0) {
                                Some((Token::Ident(_), _)) => {
                                    // SAFETY: we just checked (in above 2 lines) that the next token is an ident.
                                    let (name, span) = unsafe { self.next_ident_unchecked() };
                                    fields.push((name, span));
                                }
                                _ => todo!("implement error recovery"),
                            }
                        }
                        Some((token, span)) => {
                            self.report(Error::ExpectedFound {
                                expected: "(",
                                found: (token.to_string(), *span),
                            });
                            break ((*name, name_span), Vec::new());
                        }
                        _ => todo!("implement error recovery"),
                    }
                };
                (name, fields, args)
            }
            Some((Token::OpenParen, open_span)) => {
                let name_span = TextSpan::new(start_span.end(), open_span.start());
                self.report(Error::MissingRequiredElement("<name>", start_span));
                let (args, _) = self.func_def_args();
                (("$dummy", name_span), vec![], args)
            }
            _ => todo!("implement error recovery"),
        };
        let (body, end_span) = self.block_until_end_token();
        if fields.is_empty() {
            (
                Statement::Func {
                    name,
                    args,
                    body: Chunk {
                        captures: vec![],
                        block: body,
                    },
                },
                TextSpan::new(start_span.start(), end_span.end()),
            )
        } else {
            (
                Statement::FieldFunc {
                    table: name,
                    args,
                    fields,
                    body: Chunk {
                        captures: vec![],
                        block: body,
                    },
                },
                TextSpan::new(start_span.start(), end_span.end()),
            )
        }
    }

    // if [expr] then
    //     [block]
    // elif [expr] then
    //    [block]
    // ...
    // else
    //    [block]
    // end
    fn if_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        let (cond, cond_span) = match self.expression() {
            Some(e) => e,
            None => match self.look(0) {
                Some((Token::Then, then_span)) => {
                    let span = TextSpan::new(start_span.end(), then_span.start());
                    self.report(Error::ExpectedFound {
                        expected: "<expr>",
                        found: ("then".to_string(), *then_span),
                    });
                    (Expression::Error, span)
                }
                Some(_) => {
                    todo!("implement error recovery");
                }
                None => {
                    let expr_span = TextSpan::new(start_span.end(), self.eoi_span().end());
                    self.report(Error::UnexpectedEof("<expr>", expr_span));
                    return (
                        Statement::If {
                            cond: (Expression::Error, expr_span),
                            body: Block(vec![]),
                            elifs: vec![],
                            else_: None,
                        },
                        TextSpan::new(start_span.start(), expr_span.end()),
                    );
                }
            },
        };
        match self.look(0) {
            Some((Token::Then, _)) => {
                self.move_next();
            }
            Some((Token::Do, do_span)) => {
                self.report(Error::ExpectedFound {
                    expected: "then",
                    found: ("do".to_string(), *do_span),
                });
                self.move_next();
            }
            Some(_) => {
                todo!("implement error recovery");
            }
            None => {
                let err_span = TextSpan::new(cond_span.end(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("then", err_span));
                return (
                    Statement::If {
                        cond: (cond, cond_span),
                        body: Block(vec![]),
                        elifs: vec![],
                        else_: None,
                    },
                    TextSpan::new(start_span.start(), self.eoi_span().end()),
                );
            }
        };
        let body = {
            let mut stmts = Vec::new();
            loop {
                match self.next() {
                    Some((Token::End, end_span)) => {
                        return (
                            Statement::If {
                                cond: (cond, cond_span),
                                body: Block(stmts),
                                elifs: vec![],
                                else_: None,
                            },
                            TextSpan::new(start_span.start(), end_span.end()),
                        );
                    }
                    Some((Token::Elif, _)) => {
                        break Block(stmts);
                    }
                    Some((Token::Else, _)) => {
                        let (else_body, end_span) = self.block_until_end_token();
                        return (
                            Statement::If {
                                cond: (cond, cond_span),
                                body: Block(stmts),
                                elifs: vec![],
                                else_: Some(else_body),
                            },
                            TextSpan::new(start_span.start(), end_span.end()),
                        );
                    }
                    Some((token, span)) => {
                        let Some(stmt) = self.statement_with(token, span) else {
                            self.report(Error::UnexpectedEof("end", self.eoi_span()));
                            return (
                                Statement::If {
                                    cond: (cond, cond_span),
                                    body: Block(stmts),
                                    elifs: vec![],
                                    else_: None,
                                },
                                TextSpan::new(start_span.start(), span.start()),
                            );
                        };
                        stmts.push(stmt);
                    }
                    None => {
                        self.report(Error::UnexpectedEof("end", self.eoi_span()));
                        return (
                            Statement::If {
                                cond: (cond, cond_span),
                                body: Block(stmts),
                                elifs: vec![],
                                else_: None,
                            },
                            TextSpan::new(start_span.start(), self.eoi_span().end()),
                        );
                    }
                }
            }
        };
        let mut elifs = Vec::new();
        loop {
            let elif_cond = match self.expression() {
                Some(e) => e,
                None => match self.look(0) {
                    Some((Token::Then, then_span)) => {
                        let span = TextSpan::new(start_span.end(), then_span.start());
                        self.report(Error::ExpectedFound {
                            expected: "<expr>",
                            found: ("then".to_string(), *then_span),
                        });
                        (Expression::Error, span)
                    }
                    Some(_) => {
                        todo!("implement error recovery");
                    }
                    None => {
                        let expr_span = TextSpan::new(start_span.end(), self.eoi_span().end());
                        self.report(Error::UnexpectedEof("<expr>", expr_span));
                        return (
                            Statement::If {
                                cond: (cond, cond_span),
                                body,
                                elifs,
                                else_: None,
                            },
                            TextSpan::new(start_span.start(), expr_span.end()),
                        );
                    }
                },
            };
            match self.look(0) {
                Some((Token::Then, _)) => {
                    self.move_next();
                }
                Some((Token::Do, do_span)) => {
                    self.report(Error::ExpectedFound {
                        expected: "then",
                        found: ("do".to_string(), *do_span),
                    });
                    self.move_next();
                }
                Some(_) => {
                    todo!("implement error recovery");
                }
                None => {
                    self.report(Error::UnexpectedEof("then", self.eoi_span()));
                    return (
                        Statement::If {
                            cond: (cond, cond_span),
                            body,
                            elifs,
                            else_: None,
                        },
                        TextSpan::new(start_span.start(), self.eoi_span().end()),
                    );
                }
            };
            let elif_body = {
                let mut stmts = Vec::new();
                loop {
                    match self.next() {
                        Some((Token::End, end_span)) => {
                            elifs.push((elif_cond, Block(stmts)));
                            return (
                                Statement::If {
                                    cond: (cond, cond_span),
                                    body,
                                    elifs,
                                    else_: None,
                                },
                                TextSpan::new(start_span.start(), end_span.end()),
                            );
                        }
                        Some((Token::Elif, _)) => {
                            break Block(stmts);
                        }
                        Some((Token::Else, _)) => {
                            elifs.push((elif_cond, Block(stmts)));
                            let (else_body, end_span) = self.block_until_end_token();
                            return (
                                Statement::If {
                                    cond: (cond, cond_span),
                                    body,
                                    elifs,
                                    else_: Some(else_body),
                                },
                                TextSpan::new(start_span.start(), end_span.end()),
                            );
                        }
                        Some((token, span)) => {
                            let Some(stmt) = self.statement_with(token, span) else {
                                return (
                                    Statement::If {
                                        cond: (cond, cond_span),
                                        body,
                                        elifs,
                                        else_: None,
                                    },
                                    TextSpan::new(start_span.start(), span.start()),
                                );
                            };
                            stmts.push(stmt);
                        }
                        None => {
                            self.report(Error::UnexpectedEof("end", cond_span));
                            return (
                                Statement::If {
                                    cond: (cond, cond_span),
                                    body,
                                    elifs,
                                    else_: None,
                                },
                                TextSpan::new(start_span.start(), self.eoi_span().end()),
                            );
                        }
                    }
                }
            };
            elifs.push((elif_cond, elif_body));
        }
    }

    // for [name] in [expr] do
    //     [block]
    // end
    fn for_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        let (name, name_span) = match self.look(0) {
            Some((Token::Ident(_), _)) => {
                // SAFETY: This branch is `self.look(0) == Some(Token::Ident(_), _)`.
                let (name, span) = unsafe { self.next_ident_unchecked() };
                (name, span)
            }
            Some((Token::In, in_span)) => {
                let span = TextSpan::new(start_span.end(), in_span.start());
                self.report(Error::MissingRequiredElement("<name>", span));
                ("$dummy", span)
            }
            Some(_) => {
                todo!("implement error recovery");
            }
            None => {
                let err_span = TextSpan::new(start_span.end(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("<name>", err_span));
                return (
                    Statement::Error,
                    TextSpan::new(start_span.start(), err_span.end()),
                );
            }
        };
        let (expr, expr_span) = match self.look(0) {
            Some((Token::In, _)) => {
                self.move_next();
                match self.expression() {
                    Some(e) => e,
                    None => {
                        todo!("implement error recovery")
                    }
                }
            }
            Some((Token::Do, do_span)) => {
                let do_span = *do_span;
                self.report(Error::ExpectedFound {
                    expected: "in <expr>",
                    found: ("do".to_string(), do_span),
                });
                (
                    Expression::Error,
                    TextSpan::new(name_span.end(), do_span.start()),
                )
            }
            Some(_) => {
                todo!("implement error recovery");
            }
            None => {
                let err_span = TextSpan::new(name_span.end(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("in <expr>", err_span));
                return (
                    Statement::Error,
                    TextSpan::new(start_span.start(), err_span.end()),
                );
            }
        };
        match self.look(0) {
            Some((Token::Do, _)) => {
                self.move_next();
            }
            Some(_) => {
                todo!("implement error recovery");
            }
            None => {
                let err_span = TextSpan::new(expr_span.end(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("do", err_span));
                return (
                    Statement::For {
                        value: (name, name_span),
                        iter: (expr, expr_span),
                        body: Block(vec![]),
                    },
                    TextSpan::new(start_span.start(), err_span.end()),
                );
            }
        };
        let (body, end_span) = self.block_until_end_token();
        (
            Statement::For {
                value: (name, name_span),
                iter: (expr, expr_span),
                body,
            },
            TextSpan::new(start_span.start(), end_span.end()),
        )
    }

    // while [expr] do
    //     [block]
    // end
    fn while_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        let (expr, expr_span) = match self.expression() {
            Some((expr, span)) => (expr, span),
            None => {
                let span = TextSpan::new(start_span.start(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("<expr>", span));
                return (Statement::Error, span);
            }
        };
        match self.look(0) {
            Some((Token::Do, _)) => {
                self.move_next();
            }
            Some((Token::Then, then_span)) => {
                self.report(Error::ExpectedFound {
                    expected: "do",
                    found: ("then".to_string(), *then_span),
                });
                self.move_next();
            }
            Some(_) => {
                todo!("implement error recovery");
            }
            None => {
                let span = TextSpan::new(start_span.start(), self.eoi_span().end());
                self.report(Error::UnexpectedEof("do", span));
                return (
                    Statement::While {
                        cond: (expr, expr_span),
                        body: Block(vec![]),
                    },
                    span,
                );
            }
        }
        let (body, end_span) = self.block_until_end_token();
        (
            Statement::While {
                cond: (expr, expr_span),
                body,
            },
            TextSpan::new(start_span.start(), end_span.end()),
        )
    }

    // do
    //     [block]
    // end
    fn do_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        let (block, end_span) = self.block_until_end_token();
        (
            Statement::Do { body: block },
            TextSpan::new(start_span.start(), end_span.end()),
        )
    }

    // return
    // return [expr]
    fn return_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        if let Some((expr, expr_span)) = self.expression() {
            (
                Statement::Return {
                    value: Some((expr, expr_span)),
                },
                TextSpan::new(start_span.start(), expr_span.end()),
            )
        } else {
            (Statement::Return { value: None }, start_span)
        }
    }

    // assign:
    //   ident = [expr]
    //   [expr].ident = [expr]
    //   [expr][[expr]] = [expr]
    // call:
    //   [expr]([expr,*])
    fn assign_or_call_statement(
        &mut self,
        ident: &'src str,
        ident_span: TextSpan,
    ) -> (Statement<'src>, TextSpan) {
        if let Some((Token::Assign, _)) = self.look(0) {
            self.move_next();
            let Some((expr, expr_span)) = self.expression() else {
                todo!("implement error recovery");
            };
            return (
                Statement::Assign {
                    name: (ident, ident_span),
                    expr: (expr, expr_span),
                },
                TextSpan::new(ident_span.start(), expr_span.end()),
            );
        }
        // NOTE: From here, we have to evaluate the expression. (since we already checked that next token is not Assign, which means it's not `ident = [expr]`).
        //       To call `self.expression()` we have to move the cursor back to the ident.
        self.move_prev();
        // SAFETY: The next token is Ident because we called `self.move_prev()` above.
        //         Whenever the next token is Ident, `self.expression()` returns Some.
        let (base_expr, base_expr_span) = unsafe { self.expression().unwrap_unchecked() };
        self.expr_to_statement(base_expr, base_expr_span)
    }

    fn expr_to_statement(
        &mut self,
        base_expr: Expression<'src>,
        base_expr_span: TextSpan,
    ) -> (Statement<'src>, TextSpan) {
        match base_expr {
            Expression::IndexAccess {
                expr: (table, table_span),
                accessor: (field, field_span),
            } => self.make_field_assign_statement((*table, table_span), (*field, field_span)),
            Expression::DotAccess {
                expr: (table, table_span),
                accessor: (field, field_span),
            } => {
                let field = Expression::Primitive(Primitive::String(field.into()), field_span);
                self.make_field_assign_statement((*table, table_span), (field, field_span))
            }
            Expression::Call {
                expr: (expr, expr_span),
                args,
            } => (
                Statement::Call {
                    expr: (*expr, expr_span),
                    args,
                },
                base_expr_span,
            ),
            Expression::MethodCall {
                expr: (expr, expr_span),
                name,
                args,
            } => (
                Statement::MethodCall {
                    expr: (*expr, expr_span),
                    name,
                    args,
                },
                base_expr_span,
            ),
            _ => todo!("implement error recovery"),
        }
    }

    fn make_field_assign_statement(
        &mut self,
        (table, table_span): (Expression<'src>, TextSpan),
        (field, field_span): (Expression<'src>, TextSpan),
    ) -> (Statement<'src>, TextSpan) {
        if let Some((Token::Assign, _)) = self.look(0) {
            self.move_next();
            let Some((expr, expr_span)) = self.expression() else {
                todo!("implement error recovery");
            };
            (
                Statement::FieldAssign {
                    table: (table, table_span),
                    field: (field, field_span),
                    expr: (expr, expr_span),
                },
                TextSpan::new(table_span.start(), expr_span.end()),
            )
        } else {
            todo!("implement error recovery");
        }
    }

    fn attribute_statement(
        &mut self,
        _name: &'src str,
        _start_span: TextSpan,
    ) -> (Statement<'src>, TextSpan) {
        todo!()
    }
}
