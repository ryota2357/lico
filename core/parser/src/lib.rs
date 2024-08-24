use foundation::syntax::*;
use rowan::{GreenNode, GreenNodeBuilder, TextRange};

mod grammar;
mod parser;
mod token_set;

pub fn parse(
    source: &str,
    token_iter: impl Iterator<Item = token::Token>,
) -> (GreenNode, Vec<SyntaxError>) {
    let (kind_list, start_offsets, mut syntax_erros) = {
        let mut kind_list = Vec::new();
        let mut start_offsets = Vec::new();
        let mut syntax_errors = Vec::new();
        let mut offset = 0;
        for token in token_iter {
            let (kind, err) = convert_kind(token.kind);
            kind_list.push(kind);
            start_offsets.push(offset);
            if let Some(err) = err {
                syntax_errors.push(SyntaxError::new(
                    err.into(),
                    TextRange::at(offset.into(), token.len.into()),
                ));
            }
            offset += token.len;
        }
        (kind_list, start_offsets, syntax_errors)
    };

    let mut parser = parser::Parser::new(kind_list);
    grammar::entry(&mut parser);
    let output = parser.finish();
    let (green_node, errors) = build_tree(source, output, start_offsets);
    syntax_erros.extend(errors);
    (green_node, syntax_erros)
}

fn build_tree(
    source: &str,
    mut output: parser::Output,
    mut start_offsets: Vec<u32>,
) -> (GreenNode, Vec<SyntaxError>) {
    use parser::Event;

    struct OffsetRange {
        offsets: Vec<u32>,
        index: usize,
    }
    impl OffsetRange {
        fn current(&self) -> (u32, u32) {
            let start = self.offsets[self.index];
            let end = self.offsets[self.index + 1];
            (start, end)
        }
        fn next(&mut self) -> (u32, u32) {
            let (start, end) = self.current();
            self.index += 1;
            (start, end)
        }
    }
    start_offsets.push(source.len() as u32);
    let mut offset_range = OffsetRange {
        offsets: start_offsets,
        index: 0,
    };
    let mut builder = GreenNodeBuilder::new();
    let mut errors = Vec::new();
    let mut forward_parents = Vec::new();
    let mut range_error_start = None;

    for i in 0..output.event_count() {
        match output.take_event(i) {
            Event::StartNode {
                kind,
                forward_parent,
            } => {
                if forward_parent.is_none() {
                    builder.start_node(kind.into());
                    continue;
                }
                forward_parents.push(kind);
                let mut idx = i;
                let mut fp = forward_parent;
                while let Some(fpi) = fp {
                    idx += fpi.get() as usize;
                    fp = match output.take_event(idx) {
                        Event::StartNode {
                            kind,
                            forward_parent,
                        } => {
                            forward_parents.push(kind);
                            forward_parent
                        }
                        _ => unreachable!(),
                    };
                }
                for kind in forward_parents.drain(..).rev() {
                    builder.start_node(kind.into());
                }
            }
            Event::FinishNode => {
                builder.finish_node();
            }
            Event::Token { kind } => {
                let (start, end) = offset_range.next();
                let text = &source[start as usize..end as usize];
                builder.token(kind.into(), text);
            }
            Event::None => {}
            Event::EmptyError { message } => {
                let (start, _) = offset_range.next();
                let range = TextRange::empty(start.into());
                errors.push(SyntaxError::new(message, range));
            }
            Event::StartError => {
                let (start, _) = offset_range.current();
                range_error_start = Some(start);
            }
            Event::FinishError { message } => {
                let Some(start) = range_error_start.take() else {
                    unreachable!();
                };
                let (_, end) = offset_range.next();
                let range = TextRange::new(start.into(), end.into());
                errors.push(SyntaxError::new(message, range));
            }
        }
    }

    (builder.finish(), errors)
}

fn convert_kind(kind: token::TokenKind) -> (SyntaxKind, Option<&'static str>) {
    let mut error = None;
    let syntax_kind = match kind {
        token::TokenKind::LineComment => SyntaxKind::COMMENT,
        token::TokenKind::Whitespace => SyntaxKind::WHITESPACE,
        token::TokenKind::Int { base: _, empty_int } => {
            if empty_int {
                error = Some("Missing digits after the integer base prefix");
            }
            T![int]
        }
        token::TokenKind::Float {
            base: _,
            empty_exponent,
        } => {
            if empty_exponent {
                error = Some("Missing digits after the exponent symbol");
            }
            T![float]
        }
        token::TokenKind::String {
            terminated,
            quote_kind,
        } => {
            if !terminated {
                error = Some(match quote_kind {
                    token::QuoteKind::Single => {
                        "Missing trailing `'` symbol to terminate the string literal"
                    }
                    token::QuoteKind::Double => {
                        "Missing trailing `\"` symbol to terminate the string literal"
                    }
                });
            }
            T![string]
        }
        token::TokenKind::True => T![true],
        token::TokenKind::False => T![false],
        token::TokenKind::Nil => T![nil],
        token::TokenKind::Var => T![var],
        token::TokenKind::Func => T![func],
        token::TokenKind::If => T![if],
        token::TokenKind::Then => T![then],
        token::TokenKind::Elif => T![elif],
        token::TokenKind::Else => T![else],
        token::TokenKind::For => T![for],
        token::TokenKind::While => T![while],
        token::TokenKind::In => T![in],
        token::TokenKind::Do => T![do],
        token::TokenKind::End => T![end],
        token::TokenKind::Return => T![return],
        token::TokenKind::Break => T![break],
        token::TokenKind::Continue => T![continue],
        token::TokenKind::And => T![and],
        token::TokenKind::Or => T![or],
        token::TokenKind::Not => T![not],
        token::TokenKind::Plus => T![+],
        token::TokenKind::Minus => T![-],
        token::TokenKind::Star => T![*],
        token::TokenKind::Slash => T![/],
        token::TokenKind::Percent => T![%],
        token::TokenKind::Amp => T![&],
        token::TokenKind::Pipe => T![|],
        token::TokenKind::Caret => T![^],
        token::TokenKind::Tilde => T![~],
        token::TokenKind::Bang => T![!],
        token::TokenKind::Eq => T![=],
        token::TokenKind::Lt => T![<],
        token::TokenKind::Gt => T![>],
        token::TokenKind::Dot => T![.],
        token::TokenKind::At => T![@],
        token::TokenKind::Comma => T![,],
        token::TokenKind::Colon => T![:],
        token::TokenKind::OpenParen => T!['('],
        token::TokenKind::CloseParen => T![')'],
        token::TokenKind::OpenBrace => T!['{'],
        token::TokenKind::CloseBrace => T!['}'],
        token::TokenKind::OpenBracket => T!['['],
        token::TokenKind::CloseBracket => T![']'],
        token::TokenKind::Arrow => T![->],
        token::TokenKind::BangEq => T![!=],
        token::TokenKind::Eq2 => T![==],
        token::TokenKind::Lt2 => T![<<],
        token::TokenKind::LtEq => T![<=],
        token::TokenKind::Gt2 => T![>>],
        token::TokenKind::GtEq => T![>=],
        token::TokenKind::Dot2 => T![..],
        token::TokenKind::Ident => T![ident],
        token::TokenKind::InvalidIdent => {
            error = Some("Identifiers contains invalid characters");
            T![ident]
        }
        token::TokenKind::Unknown => SyntaxKind::ERROR,
    };
    (syntax_kind, error)
}
