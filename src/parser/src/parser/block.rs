use super::*;

impl<'tokens, 'src: 'tokens> Parser<'tokens, 'src> {
    pub fn chunk(&mut self) -> Chunk<'src> {
        let block = self.block();
        Chunk {
            captures: vec![],
            block,
        }
    }

    pub fn block(&mut self) -> Block<'src> {
        let mut statements = vec![];
        loop {
            let Some(statement) = self.statement() else {
                break;
            };
            statements.push(statement);
        }
        Block(statements)
    }

    pub fn block_until_end_token(&mut self) -> (Block<'src>, TextSpan) {
        let mut statements = Vec::new();
        let end_span = loop {
            match self.next() {
                Some((Token::End, span)) => break span,
                Some((token, span)) => {
                    let Some(statement) = self.statement_with(token, span) else {
                        todo!("implement error recovery");
                    };
                    statements.push(statement);
                }
                None => todo!("implement error recovery"),
            }
        };
        (Block(statements), end_span)
    }
}
