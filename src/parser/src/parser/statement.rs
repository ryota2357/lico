use super::*;

impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
    /// None = (Token::Error | Token::Comment)* EOF
    pub fn statement(&mut self) -> Option<(Statement<'src>, TextSpan)> {
        let Some((token, span)) = self.next() else {
            return None;
        };
        self.statement_with(token, span)
    }

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
            Token::And => todo!(),
            Token::Or => todo!(),
            Token::Not => todo!(),

            // delimiters
            Token::Comma => {
                self.report(Error::UnexpectedSymbol(",", span));
                Some((Statement::Error, span))
            }
            Token::Colon => {
                self.report(Error::UnexpectedSymbol(":", span));
                Some((Statement::Error, span))
            }
            Token::OpenParen => todo!(),
            Token::CloseParen => todo!(),
            Token::OpenBrace => todo!(),
            Token::CloseBrace => todo!(),
            Token::OpenBracket => todo!(),
            Token::CloseBracket => todo!(),

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
            self.next();
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
                            self.next();
                            let (args, _) = self.func_def_args();
                            break ((*name, name_span), args);
                        }
                        Some((Token::Dot, _)) => {
                            self.next();
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

    fn if_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        todo!()
    }

    // for [name] in [expr] do
    //     [block]
    // end
    fn for_statement(&mut self, start_span: TextSpan) -> (Statement<'src>, TextSpan) {
        todo!()
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
                self.next();
            }
            Some((Token::Then, then_span)) => {
                self.report(Error::ExpectedFound {
                    expected: "do",
                    found: ("then".to_string(), *then_span),
                });
                self.next();
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
        todo!()
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
    //   name = [expr]
    //   name.[access] = [expr]
    //   name[[expr]] = [expr]
    // call:
    //   name([expr])
    //   name.[access]([expr])
    //   name[[expr]]([expr])
    fn assign_or_call_statement(
        &mut self,
        name: &'src str,
        name_span: TextSpan,
    ) -> (Statement<'src>, TextSpan) {
        let accesser = {
            let mut accesser = Vec::new();
            loop {
                match self.look(0) {
                    Some((Token::Dot, _)) => {
                        self.next();
                        match self.look(0) {
                            Some((Token::Ident(name), span)) => {
                                accesser.push((
                                    Expression::Primitive(Primitive::String((*name).into()), *span),
                                    *span,
                                ));
                                self.next();
                            }
                            Some(_) => todo!("implement error recovery"),
                            None => {
                                let eoi_span = self.eoi_span();
                                let span = TextSpan::new(name_span.start(), self.eoi_span().end());
                                self.report(Error::UnexpectedEof("<key>", span));
                                return (
                                    Statement::Assign {
                                        name: (name, name_span),
                                        accesser,
                                        expr: (Expression::Error, eoi_span),
                                    },
                                    TextSpan::new(name_span.start(), eoi_span.end()),
                                );
                            }
                        }
                    }
                    Some((Token::OpenBracket, _)) => {
                        self.next();
                        if let Some((expr, span)) = self.expression() {
                            accesser.push((expr, span));
                        } else {
                            todo!("implement error recovery");
                        }
                        match self.look(0) {
                            Some((Token::CloseBracket, _)) => {
                                self.next();
                            }
                            Some((Token::Assign, assign_span)) => {
                                let span = TextSpan::new(
                                    if accesser.is_empty() {
                                        name_span.end()
                                    } else {
                                        accesser.last().unwrap().1.end()
                                    },
                                    assign_span.start(),
                                );
                                self.report(Error::MissingRequiredElement("]", span));
                                break;
                            }
                            Some(_) => {
                                todo!("implement error recovery");
                            }
                            None => {
                                let span = TextSpan::new(name_span.start(), self.eoi_span().end());
                                self.report(Error::UnexpectedEof("]", span));
                                return (
                                    Statement::Assign {
                                        name: (name, name_span),
                                        accesser,
                                        expr: (Expression::Error, TextSpan::at(name_span.end(), 0)),
                                    },
                                    span,
                                );
                            }
                        }
                    }
                    _ => break,
                }
            }
            accesser
        };
        match self.next() {
            Some((Token::Assign, _)) => {
                let Some((expr, expr_span)) = self.expression() else {
                    panic!("implement error recovery");
                };
                (
                    Statement::Assign {
                        name: (name, name_span),
                        accesser,
                        expr: (expr, expr_span),
                    },
                    TextSpan::new(name_span.start(), expr_span.end()),
                )
            }
            Some((Token::OpenParen, _)) => {
                let (args, close_span) = self.func_call_args();
                (
                    Statement::Call {
                        expr: (Expression::Local(name, name_span), name_span),
                        accesser,
                        args,
                    },
                    TextSpan::new(name_span.start(), close_span.end()),
                )
            }
            _ => todo!("implement error recovery"),
        }
    }

    fn attribute_statement(
        &mut self,
        name: &'src str,
        start_span: TextSpan,
    ) -> (Statement<'src>, TextSpan) {
        todo!()
    }
}
