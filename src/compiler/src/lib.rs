#![allow(dead_code)]

pub mod error;
use error::Error;

type Span = std::ops::Range<usize>;
type Result<T> = std::result::Result<T, Error>;

mod tools;
use tools::*;

mod block;
use block::*;
mod expression;
mod statement;

pub fn compile<'src>(program: &'src parser::Program<'src>) -> Result<Vec<vm::code::Code>> {
    use vm::code::BuiltinInstr;

    let mut fragment = Fragment::new();
    let mut context = Context::new();
    for (capture, span) in program.body.captures.iter() {
        match *capture {
            "print" => {
                context.add_variable("print");
                fragment.append_many([
                    ICode::BeginFuncCreation,
                    ICode::AddArgument(()),
                    ICode::LoadLocal(VariableId::new_manual(0)),
                    ICode::Builtin(BuiltinInstr::Write, 1),
                    ICode::Builtin(BuiltinInstr::Flush, 0),
                    ICode::LoadNil,
                    ICode::Return,
                    ICode::EndFuncCreation,
                    ICode::MakeLocal,
                ]);
            }
            "println" => {
                context.add_variable("println");
                fragment.append_many([
                    ICode::BeginFuncCreation,
                    ICode::AddArgument(()),
                    ICode::LoadLocal(VariableId::new_manual(0)),
                    ICode::LoadString("\n".to_string()),
                    ICode::Builtin(BuiltinInstr::Write, 2),
                    ICode::Builtin(BuiltinInstr::Flush, 0),
                    ICode::LoadNil,
                    ICode::Return,
                    ICode::EndFuncCreation,
                    ICode::MakeLocal,
                ]);
            }
            "require" => {
                unimplemented!("require")
            }
            name => {
                return Err(Error::undefined_variable(name.to_string(), span.clone()));
            }
        }
    }
    fragment.append_compile(&program.body.block, &mut context)?;
    if !matches!(fragment.last(), Some(ICode::Return)) {
        fragment.append_many([ICode::LoadNil, ICode::Return]);
    }

    Ok(fragment.into_code())
}
