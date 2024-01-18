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
