use super::*;

pub(crate) fn compile_function(func: &ir::FunctionKey, fragment: &mut Fragment, ctx: &mut Context) {
    use ICodeSource::*;

    let func_capture = ctx.capture_db.get_capture(func);

    let (func_fragment, func_param_len) = {
        let (func_params, func_effects) = ctx.strage.get(func);

        let mut fragment = Fragment::new();
        let mut ctx = Context::new_with(ctx);

        for capture in func_capture.iter() {
            ctx.add_local(capture);
        }

        let mut param_len = 0;
        for (_, param) in func_params {
            ctx.add_local(param.text());
            param_len += 1;
        }
        assert!(param_len <= u8::MAX as u32);

        for (_, effect) in func_effects {
            fragment.append_compile(&effect, &mut ctx);
        }

        (fragment, param_len)
    };

    let func_id = ctx.add_function(func_fragment);
    fragment
        .append_many([
            BeginFuncSection,
            FuncSetProperty(func_param_len as u8, func_id),
        ])
        .append_many(func_capture.iter().map(|name| {
            let local_id = ctx.resolve_local(name);
            FuncAddCapture(local_id)
        }))
        .append(EndFuncSection);
}
