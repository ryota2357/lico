use parser::tree::*;
use vm::code::{BuiltinInstr, Code};

mod fragment;
use fragment::Fragment;

mod context;
use context::Context;

pub mod error;
use error::Error;

type Span = std::ops::Range<usize>;
type Result<T> = std::result::Result<T, Error>;

trait Compilable<'node, 'src: 'node> {
    fn compile(&'node self, fragment: &mut Fragment<'src>) -> Result<()>;
}

trait ContextCompilable<'node, 'src: 'node> {
    fn compile(&'node self, fragment: &mut Fragment<'src>, context: &mut Context) -> Result<()>;
}

mod block;
mod expression;
mod statement;

pub fn compile<'a>(program: &'a Program<'a>) -> Result<Vec<Code<'a>>> {
    let mut fragment = Fragment::new();

    for capture in program.body.captures.iter() {
        match *capture {
            "print" => {
                fragment.append_many([
                    Code::BeginFuncCreation,
                    Code::AddArgument("value"),
                    Code::LoadLocal("value"),
                    Code::Builtin(BuiltinInstr::Write, 1),
                    Code::Builtin(BuiltinInstr::Flush, 0),
                    Code::LoadNil,
                    Code::Return,
                    Code::EndFuncCreation,
                    Code::MakeLocal("print"),
                ]);
            }
            "println" => {
                fragment.append_many([
                    Code::BeginFuncCreation,
                    Code::AddArgument("value"),
                    Code::LoadLocal("value"),
                    Code::LoadString("\n".to_string()),
                    Code::Builtin(BuiltinInstr::Write, 2),
                    Code::Builtin(BuiltinInstr::Flush, 0),
                    Code::LoadNil,
                    Code::Return,
                    Code::EndFuncCreation,
                    Code::MakeLocal("println"),
                ]);
            }
            "require" => {
                unimplemented!("require")
            }
            _ => { /* TODO: warning or ... */ }
        }
    }

    let eob = block::compile_statements(
        program.body.block.iter(),
        &mut fragment,
        &mut Context::new(),
    )?;

    match eob {
        (block::ExitControll::Return, _) => {}
        (block::ExitControll::None, _) => {
            fragment.append_many([Code::LoadNil, Code::Return]);
        }
        (block::ExitControll::Break, span) => return Err(Error::no_loop_to_break(span)),
        (block::ExitControll::Continue, span) => return Err(Error::no_loop_to_continue(span)),
    }

    Ok(fragment.into_code())
}
