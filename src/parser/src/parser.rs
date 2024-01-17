use super::*;

mod block;
mod expression;
mod statement;
mod util;

pub struct Parser<'tokens, 'src: 'tokens>(internal::ParserCore<'tokens, 'src>);

impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
    pub fn parse(tokens: &'tokens [(Token<'src>, TextSpan)]) -> (Program<'src>, Vec<Error>) {
        let mut parser = Self(internal::ParserCore::new(tokens));
        let program = parser.program();
        let errors = parser.done();
        (program, errors)
    }

    pub fn program(&mut self) -> Program<'src> {
        let chunk = self.chunk();
        Program {
            attributes: vec![],
            body: chunk,
        }
    }
}

mod internal {
    use super::*;

    pub struct ParserCore<'tokens, 'src: 'tokens> {
        tokens: &'tokens [(Token<'src>, TextSpan)],
        index: usize,
        errors: Vec<Error>,
        eoi: u32,
    }

    // TODO: move_next() とかで、コメントをスキップするようにする。
    //       Vecを新たに作ったり、move_next()を呼ぶ前にコメントをチェックするのは嫌なので、
    //       前計算で処理しておきたい、移動量をメモっておけば良さそう。NonZeroUsizeとか使えば良いかも。
    impl<'tokens, 'src: 'tokens> ParserCore<'tokens, 'src> {
        pub fn new(tokens: &'tokens [(Token<'src>, TextSpan)]) -> Self {
            Self {
                tokens,
                index: 0,
                errors: Vec::new(),
                eoi: tokens.last().map_or(0, |(_, span)| span.end()),
            }
        }
    }

    impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
        #[inline]
        #[must_use]
        pub fn done(self) -> Vec<Error> {
            self.0.errors
        }

        #[inline]
        pub fn look(&self, i: isize) -> Option<&(lexer::Token<'_>, lexer::TextSpan)> {
            let index = self.0.index as isize + i;
            debug_assert!(index >= 0);
            self.0.tokens.get(index as usize)
        }

        #[must_use]
        pub fn next(&mut self) -> Option<(&'tokens Token<'src>, TextSpan)> {
            if self.0.index < self.0.tokens.len() {
                let (token, span) = &self.0.tokens[self.0.index];
                self.0.index += 1;
                Some((token, *span))
            } else {
                None
            }
        }

        #[inline]
        pub fn move_next(&mut self) {
            debug_assert!(self.0.index < self.0.tokens.len());
            self.0.index += 1;
        }

        #[inline]
        pub fn move_prev(&mut self) {
            debug_assert!(self.0.index > 0);
            self.0.index -= 1;
        }

        #[inline]
        pub fn report(&mut self, error: Error) {
            self.0.errors.push(error);
        }

        #[inline]
        pub fn eoi_span(&self) -> TextSpan {
            TextSpan::new(self.0.eoi, self.0.eoi)
        }
    }
}
