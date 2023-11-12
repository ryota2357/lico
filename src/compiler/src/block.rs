use super::*;

impl<'a> ContextCompilable<'a> for Block<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>, context: &mut Context) {
        context.start_block();
        for statement in self.iter() {
            fragment.append_compile_with_context(statement, context);
            if matches!(
                statement,
                Statement::Control(
                    ControlStatement::Return { .. }
                        | ControlStatement::Continue
                        | ControlStatement::Break
                )
            ) {
                break;
            }
        }
        let drop_count = context.get_block_local_count().unwrap();
        if drop_count > 0 {
            fragment.append(Code::DropLocal(drop_count));
        }
        context.end_block();
    }
}

impl<'a> Compilable<'a> for Chunk<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>) {
        let mut context = Context::new();

        fragment.append_many(
            self.captures
                .iter()
                .map(|capture| Code::AddCapture(capture)),
        );
        for statement in self.body.iter() {
            fragment.append_compile_with_context(statement, &mut context);
            if matches!(
                statement,
                Statement::Control(ControlStatement::Return { .. })
            ) {
                return;
            }
        }
        fragment.append_many([Code::LoadNil, Code::Return]);
    }
}
