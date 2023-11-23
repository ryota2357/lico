use super::*;

pub(super) enum ExitControll {
    Return,
    Break,
    Continue,
    None,
}

pub(super) fn compile_statements<'node, 'src: 'node>(
    statements: impl IntoIterator<Item = &'node (Statement<'src>, Span)>,
    fragment: &mut Fragment<'src>,
    context: &mut Context,
) -> Result<(ExitControll, Span)> {
    let mut last_span = &(0..0);
    for statement in statements {
        fragment.append_compile_with_context(statement, context)?;
        match statement {
            (Statement::Control(ControlStatement::Return { .. }), span) => {
                return Ok((ExitControll::Return, span.clone()));
            }
            (Statement::Control(ControlStatement::Continue), span) => {
                return Ok((ExitControll::Continue, span.clone()));
            }
            (Statement::Control(ControlStatement::Break), span) => {
                return Ok((ExitControll::Break, span.clone()));
            }
            (_, span) => {
                last_span = span;
            }
        }
    }
    Ok((ExitControll::None, last_span.clone()))
}

impl<'node, 'src: 'node> ContextCompilable<'node, 'src> for Block<'src> {
    fn compile(&'node self, fragment: &mut Fragment<'src>, context: &mut Context) -> Result<()> {
        context.start_block();
        let end = compile_statements(self.iter(), fragment, context)?;
        if let (ExitControll::None, _) = end {
            let drop_count = context.get_block_local_count().unwrap();
            if drop_count > 0 {
                fragment.append(Code::DropLocal(drop_count));
            }
        }
        context.end_block();
        Ok(())
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for Chunk<'src> {
    fn compile(&'node self, fragment: &mut Fragment<'src>) -> Result<()> {
        fragment.append_many(
            self.captures
                .iter()
                .map(|capture| Code::AddCapture(capture)),
        );
        let end = compile_statements(self.block.iter(), fragment, &mut Context::new())?;

        match end {
            (ExitControll::Return, _) => Ok(()),
            (ExitControll::None, _) => {
                fragment.append_many([Code::LoadNil, Code::Return]);
                Ok(())
            }
            (ExitControll::Break, span) => Err(Error::no_loop_to_break(span)),
            (ExitControll::Continue, span) => Err(Error::no_loop_to_continue(span)),
        }
    }
}
