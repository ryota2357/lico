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
        RustFunction::new(1, |mut args| {
            print!("{}", args.next().unwrap());
            Ok(Object::Nil)
        }),
    ),
    (
        "println",
        RustFunction::new(1, |mut args| {
            println!("{}", args.next().unwrap());
            Ok(Object::Nil)
        }),
    ),
];

pub fn compile(module: &ir::Module) -> il::Module {
    let (capture_db, used_default) = database::FunctionCapture::build_with(
        module,
        DEFAULT_FUNCTIONS.iter().map(|(name, _)| *name),
    );

    let mut ctx = Context::new(module.strage(), &capture_db);
    let mut fragment = Fragment::new();
    let mut default_rfns = Vec::new();
    for (name, f) in DEFAULT_FUNCTIONS.iter() {
        if used_default.contains(name) {
            ctx.add_local(name);
            default_rfns.push((*name, *f));
        }
    }
    fragment.append_compile(module.effects(), &mut ctx);
    let (codes, infos) = ctx.finish_with(fragment);
    il::Module::new(
        il::Executable::new(codes),
        default_rfns.into_boxed_slice(),
        infos,
    )
}
