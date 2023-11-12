use parser::tree::*;
use vm::code::{BuiltinInstr, Code};

mod fragment;
use fragment::Fragment;

mod context;
use context::Context;

trait Compilable<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>);
}

trait ContextCompilable<'a> {
    fn compile(&self, fragment: &mut Fragment<'a>, context: &mut Context);
}

mod block;
mod expression;
mod statement;

pub fn compile<'a>(program: &Program<'a>) -> Vec<Code<'a>> {
    let mut fragment = Fragment::new();

    for capture in program.body.captures.iter() {
        match *capture {
            "print" => {
                fragment.append_many([
                    Code::BeginFuncCreation,
                    Code::AddArgument("value"),
                    Code::LoadLocal("value"),
                    Code::LoadString("\n".to_string()),
                    Code::Builtin(BuiltinInstr::Write, 2),
                    Code::Builtin(BuiltinInstr::Flush, 0),
                    Code::EndFuncCreation,
                    Code::MakeLocal("print"),
                ]);
            }
            "require" => {
                unimplemented!("require")
            }
            _ => {}
        }
    }
    fragment.append_compile(&program.body);

    fragment.into_code()
}
