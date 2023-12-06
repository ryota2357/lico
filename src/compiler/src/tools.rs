use super::*;

mod context;
pub use context::*;

mod fragment;
pub use fragment::*;

mod icode;
pub use icode::*;

pub trait Compilable<'node, 'src: 'node> {
    fn compile(&'node self, fragment: &mut Fragment, context: &mut Context<'src>) -> Result<()>;
}
