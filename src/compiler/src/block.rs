use super::*;

pub(super) enum ExitControll {
    Return,
    Break,
    Continue,
    None,
}

pub(super) fn compile_statements<'node, 'src: 'node>(
    statements: impl IntoIterator<Item = &'node Statement<'src>>,
    fragment: &mut Fragment<'src>,
    context: &mut Context,
) -> ExitControll {
    for statement in statements.into_iter() {
        fragment.append_compile_with_context(statement, context);
        match statement {
            Statement::Control(ControlStatement::Return { .. }) => {
                return ExitControll::Return;
            }
            Statement::Control(ControlStatement::Continue) => {
                return ExitControll::Continue;
            }
            Statement::Control(ControlStatement::Break) => {
                return ExitControll::Break;
            }
            _ => {}
        }
    }
    ExitControll::None
}

impl<'node, 'src: 'node> ContextCompilable<'node, 'src> for Block<'src> {
    fn compile(&'node self, fragment: &mut Fragment<'src>, context: &mut Context) {
        context.start_block();
        let end = compile_statements(self.iter().map(|s| &s.0), fragment, context);
        if let ExitControll::None = end {
            let drop_count = context.get_block_local_count().unwrap();
            if drop_count > 0 {
                fragment.append(Code::DropLocal(drop_count));
            }
        }
        context.end_block();
    }
}

impl<'node, 'src: 'node> Compilable<'node, 'src> for Chunk<'src> {
    fn compile(&'node self, fragment: &mut Fragment<'src>) {
        fragment.append_many(
            self.captures
                .iter()
                .map(|capture| Code::AddCapture(capture)),
        );
        let end = compile_statements(
            self.block.iter().map(|s| &s.0),
            fragment,
            &mut Context::new(),
        );

        match end {
            ExitControll::Return => {}
            ExitControll::None => {
                fragment.append_many([Code::LoadNil, Code::Return]);
            }
            ExitControll::Break | ExitControll::Continue => panic!(),
        }
    }
}
