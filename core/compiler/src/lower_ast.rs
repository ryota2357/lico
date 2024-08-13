use compact_str::CompactString;
use foundation::{
    ir,
    syntax::{ast, ast::AstNode, SyntaxError, SyntaxNode, SyntaxToken},
};

mod effect;
use effect::effect;

mod value;
use value::value;

mod context;
use context::{Context, ScopeKind};

pub fn lower(program: ast::Program) -> (ir::Module, Vec<SyntaxError>) {
    let mut ctx = Context::new();
    let effects: Vec<_> = program.into_lowered(&mut ctx);
    let key = ctx.builder.add_effects(effects);
    let (mod_builder, errors) = ctx.finish();
    let module = mod_builder.finish_with(key);
    (module, errors)
}

trait IntoLowered<T> {
    fn into_lowered(self, ctx: &mut Context) -> T;
}

impl IntoLowered<Vec<(SyntaxNode, ir::Effect)>> for ast::Program {
    fn into_lowered(self, ctx: &mut Context) -> Vec<(SyntaxNode, ir::Effect)> {
        let mut effects = Vec::new();
        for stmt in self.statements() {
            let sn = stmt.syntax().clone();
            let effect = effect(ctx, stmt);
            effects.push((sn, effect));
        }
        effects
    }
}

impl
    IntoLowered<(
        Vec<(SyntaxNode, ir::Effect)>,
        Option<(SyntaxNode, ir::Value)>,
    )> for ast::Program
{
    fn into_lowered(
        self,
        ctx: &mut Context,
    ) -> (
        Vec<(SyntaxNode, ir::Effect)>,
        Option<(SyntaxNode, ir::Value)>,
    ) {
        let mut stmt_iter = self.statements();
        let Some(mut last_stmt) = stmt_iter.next() else {
            return (Vec::new(), None);
        };
        let mut effects = Vec::new();
        for next in stmt_iter {
            let sn = last_stmt.syntax().clone();
            effects.push((sn, effect(ctx, last_stmt)));
            last_stmt = next;
        }
        let value = match last_stmt {
            ast::Statement::Expr(expt_stmt) => expt_stmt.expr().map(|expr| {
                let syntax = expr.syntax().clone();
                (syntax, value(ctx, expr))
            }),
            _ => {
                let syntax = last_stmt.syntax().clone();
                effects.push((syntax, effect(ctx, last_stmt)));
                None
            }
        };
        (effects, value)
    }
}

impl IntoLowered<Vec<(SyntaxNode, ir::Value)>> for ast::ArgList {
    fn into_lowered(self, ctx: &mut Context) -> Vec<(SyntaxNode, ir::Value)> {
        let mut args = Vec::new();
        for arg in self.args() {
            let sn = arg.syntax().clone();
            let value = value(ctx, arg);
            args.push((sn, value));
        }
        args
    }
}

impl IntoLowered<Vec<(SyntaxToken, ir::Symbol)>> for ast::ParamList {
    fn into_lowered(self, ctx: &mut Context) -> Vec<(SyntaxToken, ir::Symbol)> {
        let mut params = Vec::new();
        let scope = ctx.scope_index();
        for param in self.params() {
            let Some(token) = param.ident_token() else {
                continue;
            };
            let text = CompactString::from(token.text());
            let symbol = ir::Symbol::new(text, scope);
            params.push((token, symbol));
        }
        params
    }
}
