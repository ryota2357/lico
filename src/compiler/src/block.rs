use super::*;

impl<'node, 'src: 'node> Compilable<'node, 'src> for Block<'src> {
    fn compile(&'node self, fragment: &mut Fragment, context: &mut Context<'src>) -> Result<()> {
        context.begin_block();
        for statement in self.iter() {
            statement.compile(fragment, context)?;
        }
        if !matches!(fragment.last(), Some(ICode::Return)) {
            let drop_count = context.get_block_local_count();
            if drop_count > 0 {
                fragment.append(ICode::DropLocal(drop_count));
            }
        }
        context.end_block();
        Ok(())
    }
}

pub mod util {
    use super::*;
    use vm::code::ArgumentKind;

    pub fn append_func_creation_fragment<'node, 'src: 'node>(
        fragment: &mut Fragment,
        chunk: &'node Chunk<'src>,
        args: &'node [(FunctArgAnnotation, &'src str, TextSpan)],
        context: &mut Context<'src>,
    ) -> Result<()> {
        let add_capture = chunk
            .captures
            .iter()
            .map(|(name, span)| {
                let id = context
                    .resolve_variable(name)
                    .ok_or_else(|| Error::undefined_variable(name.to_string(), *span))?;
                Ok(ICode::AddCapture(id))
            })
            .collect::<Result<Vec<_>>>()?;
        let add_argument = args.iter().map(|_| ICode::AddArgument(ArgumentKind::Copy));
        let block_fragment = {
            let mut context = Context::new();
            context.begin_block();
            context.add_variable_many(chunk.captures.iter().map(|(name, _)| *name));
            context.add_variable_many(args.iter().map(|(_, name, _)| *name));
            let mut fragment = Fragment::with_compile(&chunk.block, &mut context)?;
            if !matches!(fragment.last(), Some(ICode::Return)) {
                fragment.append_many([ICode::LoadNil, ICode::Return]);
            }
            fragment
        };
        fragment
            .append(ICode::BeginFuncCreation)
            .append_many(add_capture)
            .append_many(add_argument)
            .append_fragment(block_fragment)
            .append(ICode::EndFuncCreation);
        Ok(())
    }
}
