use foundation::{
    il, ir,
    object::{Object, RustFunction},
};

mod context;
use context::*;

mod fragment;
use fragment::*;

mod icodesource;
use icodesource::*;

mod compile_utils;

mod compilable_impl_effect;
mod compilable_impl_value;

use crate::database;

trait Compilable<'node, 'src: 'node> {
    fn compile(&'node self, fragment: &mut Fragment, ctx: &mut Context<'src>);
}

// TODO: The name of the default capture names are taken as an argument to `compile`. (Not defined here).
const DEFAULT_FUNCTIONS: [(&str, RustFunction); 2] = [
    (
        "print",
        RustFunction::new(1, |args: &[Object]| {
            print!("{:?}", args[0]);
            Ok(Object::Nil)
        }),
    ),
    (
        "println",
        RustFunction::new(1, |args: &[Object]| {
            println!("{:?}", args[0]);
            Ok(Object::Nil)
        }),
    ),
];

pub fn compile(module: &ir::Module) -> il::Module {
    let mut capture_db = database::FunctionCapture::new();
    for (name, _) in DEFAULT_FUNCTIONS.iter() {
        capture_db.insert(module, *name);
    }
    capture_db.build_with(module);

    let mut ctx = Context::new(&module.strage, &capture_db);
    let mut fragment = Fragment::new();
    let mut default_rfns = Vec::new();
    for (name, func) in DEFAULT_FUNCTIONS.iter() {
        let mut is_used = false;
        for (_, capture) in capture_db.iter_captures() {
            if capture.contains(*name) {
                is_used = true;
                break;
            }
        }
        if is_used {
            default_rfns.push((*name, *func));
            ctx.add_local(name);
        }
    }
    fragment.append_compile(&module.effects, &mut ctx);
    let (codes, infos) = ctx.finish_with(fragment);
    il::Module::new(
        il::Executable::new(codes),
        default_rfns.into_boxed_slice(),
        infos,
    )
}
