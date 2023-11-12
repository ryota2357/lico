use super::*;

pub(super) enum ExitControll {
    Return,
    Break,
    Continue,
    None,
}

pub(super) fn compile_statements<'a>(
    statements: &[Statement<'a>],
    fragment: &mut Fragment<'a>,
    context: &mut Context,
) -> ExitControll {
    for statement in statements.iter() {
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

impl<'a> ContextCompilable<'a> for Block<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>, context: &mut Context) {
        context.start_block();
        let end = compile_statements(self, fragment, context);
        if let ExitControll::None = end {
            let drop_count = context.get_block_local_count().unwrap();
            if drop_count > 0 {
                fragment.append(Code::DropLocal(drop_count));
            }
        }
        context.end_block();
    }
}

impl<'a> Compilable<'a> for Chunk<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>) {
        fragment.append_many(
            self.captures
                .iter()
                .map(|capture| Code::AddCapture(capture)),
        );
        let end = compile_statements(self, fragment, &mut Context::new());

        match end {
            ExitControll::Return => {}
            ExitControll::None => {
                fragment.append_many([Code::LoadNil, Code::Return]);
            }
            ExitControll::Break | ExitControll::Continue => panic!(),
        }
    }
}
