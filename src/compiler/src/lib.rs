use parser::tree::*;
use vm::code::{BuiltinInstr, Code};

mod fragment;
use fragment::Fragment;

mod context;
use context::Context;

trait Compilable<'node, 'src: 'node> {
    fn compile(&'node self, fragment: &mut Fragment<'src>);
}

trait ContextCompilable<'node, 'src: 'node> {
    fn compile(&'node self, fragment: &mut Fragment<'src>, context: &mut Context);
}

mod block;
mod expression;
mod statement;

pub fn compile<'a>(program: &'a Program<'a>) -> Vec<Code<'a>> {
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
                    Code::LoadNil,
                    Code::Return,
                    Code::EndFuncCreation,
                    Code::MakeLocal("print"),
                ]);
            }
            "require" => {
                unimplemented!("require")
            }
            _ => { /* TODO: warning or ... */ }
        }
    }

    let eob = block::compile_statements(
        program.body.iter().map(|s| &s.0),
        &mut fragment,
        &mut Context::new(),
    );

    match eob {
        block::ExitControll::Return => {}
        block::ExitControll::None => {
            fragment.append_many([Code::LoadNil, Code::Return]);
        }
        block::ExitControll::Break | block::ExitControll::Continue => panic!(),
    }

    fragment.into_code()
}
