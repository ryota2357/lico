use super::*;

impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
    pub fn func_def_args(&mut self) -> (Vec<(FunctArgAnnotation, &'src str, TextSpan)>, TextSpan) {
        debug_assert!(matches!(self.look(-1), Some((Token::OpenParen, _))));

        let mut args = Vec::<(FunctArgAnnotation, &'src str, TextSpan)>::new();
        let mut annotation = None;
        let close_span = loop {
            let Some((peek, peek_span)) = self.look(0) else {
                let eoi_span = self.eoi_span();
                self.report(Error::UnexpectedEof(")", eoi_span));
                break eoi_span;
            };
            match peek {
                Token::Ident(_) => {
                    // SAFETY: `peek` is self.look(0) and it is `Token::Ident`.
                    let (name, name_span) = unsafe { self.next_ident_unchecked() };
                    let annotation = annotation.take().unwrap_or(FunctArgAnnotation::None);
                    args.push((annotation, name, name_span));
                }
                tok @ (Token::In | Token::Ref) => {
                    if annotation.is_some() {
                        self.report(Error::ExpectedFound {
                            expected: "<name>",
                            found: (peek.to_string(), *peek_span),
                        });
                    } else {
                        annotation = Some(match tok {
                            Token::In => FunctArgAnnotation::In,
                            Token::Ref => FunctArgAnnotation::Ref,
                            _ => unreachable!(),
                        });
                    }
                    self.next();
                }
                Token::CloseParen => {
                    let close_span = *peek_span;
                    self.next();
                    break close_span;
                }
                _ => todo!("implement error recovery"),
            }
            match self.look(0) {
                Some((Token::Comma, _)) => {
                    self.next();
                }
                Some((Token::CloseParen, span)) => {
                    let close_span = *span;
                    self.next();
                    break close_span;
                }
                _ => todo!("implement error recovery"),
            }
        };
        (args, close_span)
    }

    pub fn func_call_args(&mut self) -> (Vec<(Expression<'src>, TextSpan)>, TextSpan) {
        debug_assert!(matches!(self.look(-1), Some((Token::OpenParen, _))));

        if let Some((Token::CloseParen, _)) = self.look(0) {
            let (_, close_span) = unsafe { self.next().unwrap_unchecked() };
            return (Vec::new(), close_span);
        }

        let mut args = Vec::new();
        let close_span = loop {
            let Some((arg, arg_span)) = self.expression() else {
                match self.look(0) {
                    Some((Token::Comma, span)) => {
                        self.report(Error::UnexpectedSymbol(",", *span));
                        self.next();
                        continue;
                    }
                    Some(_) => {
                        todo!("implement error recovery");
                    }
                    None => {
                        let eoi_span = self.eoi_span();
                        self.report(Error::UnexpectedEof(")", eoi_span));
                        break eoi_span;
                    }
                }
            };
            args.push((arg, arg_span));
            let Some((delim, delim_span)) = self.look(0) else {
                let eoi_span = self.eoi_span();
                self.report(Error::UnexpectedEof(")", eoi_span));
                break eoi_span;
            };
            match delim {
                Token::Comma => {
                    self.next();
                }
                Token::CloseParen => {
                    let close_span = *delim_span;
                    self.next();
                    break close_span;
                }
                _ => todo!("implement error recovery"),
            }
        };
        (args, close_span)
    }

    /// This function is needed to avoid a lifetime error.
    ///
    /// ```ignore
    /// match self.look(0) {
    ///    Some((Token::Ident(name), span)) => {
    ///        // For here, `name` and `span` is 'tokens lifetime.
    ///        // But in some cases, we need 'src lifetime.
    ///        // So we use `next_ident_unchecked` to get 'src lifetime.
    ///    }
    /// }
    /// ```
    pub unsafe fn next_ident_unchecked(&mut self) -> (&'src str, TextSpan) {
        let next = self.next().unwrap_unchecked();
        debug_assert!(matches!(next.0, Token::Ident(_)));
        let wrapped = match next {
            (Token::Ident(name), span) => Some((*name, span)),
            _ => None,
        };
        wrapped.unwrap_unchecked()
    }
}
