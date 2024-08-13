use core::mem::forget;
use foundation::{
    ir::{ModuleBuilder, ScopeIndex},
    syntax::{SyntaxError, TextRange},
};
use std::borrow::Cow;

pub(super) struct Context {
    pub(super) builder: ModuleBuilder,
    errors: Vec<SyntaxError>,
    current_scope: ScopeIndex,
    next_scope: ScopeIndex,
    in_loop_scope: bool,
}

impl Context {
    pub(super) const fn new() -> Self {
        let current_scope = ScopeIndex::new();
        Self {
            builder: ModuleBuilder::new(),
            errors: Vec::new(),
            current_scope,
            next_scope: current_scope.make_next(),
            in_loop_scope: false,
        }
    }

    pub(super) fn finish(self) -> (ModuleBuilder, Vec<SyntaxError>) {
        let Context {
            builder,
            errors,
            current_scope,
            next_scope: _,
            in_loop_scope,
        } = self;
        debug_assert_eq!(current_scope.as_u32(), 1);
        debug_assert!(!in_loop_scope);
        (builder, errors)
    }

    pub(super) fn is_in_loop(&self) -> bool {
        self.in_loop_scope
    }

    pub(super) fn scope_index(&self) -> ScopeIndex {
        self.current_scope
    }

    pub(super) fn push_error(&mut self, message: impl Into<Cow<'static, str>>, range: TextRange) {
        self.errors.push(SyntaxError::new(message.into(), range));
    }

    pub(super) fn start_scope(&mut self, kind: ScopeKind) -> ScopeMarker {
        let current = self.current_scope;
        let in_loop = self.in_loop_scope;
        self.current_scope = self.next_scope;
        self.next_scope = self.next_scope.make_next();
        match kind {
            ScopeKind::Nest => ScopeMarker { current, in_loop },
            ScopeKind::New => {
                self.in_loop_scope = false;
                ScopeMarker { current, in_loop }
            }
            ScopeKind::Loop => {
                self.in_loop_scope = true;
                ScopeMarker { current, in_loop }
            }
        }
    }
}

pub(super) enum ScopeKind {
    Nest,
    New,
    Loop,
}

#[must_use]
pub(super) struct ScopeMarker {
    current: ScopeIndex,
    in_loop: bool,
}

impl ScopeMarker {
    pub(super) fn finish(self, ctx: &mut Context) {
        ctx.current_scope = self.current;
        ctx.in_loop_scope = self.in_loop;
        forget(self);
    }
}

impl Drop for ScopeMarker {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("ScopeMarker must be completed with finish() method");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_scope() {
        let mut ctx = Context::new();
        assert_eq!(ctx.scope_index().as_u32(), 1);

        let scope1 = ctx.start_scope(ScopeKind::New);
        assert_eq!(ctx.scope_index().as_u32(), 2);

        let scope2 = ctx.start_scope(ScopeKind::Nest);
        assert_eq!(ctx.scope_index().as_u32(), 3);

        scope2.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 2);

        scope1.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);
    }

    #[test]
    fn nested_scope2() {
        let mut ctx = Context::new();
        assert_eq!(ctx.scope_index().as_u32(), 1);

        let nest_scope = ctx.start_scope(ScopeKind::New);
        assert_eq!(ctx.scope_index().as_u32(), 2);
        nest_scope.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);

        let nest_scope = ctx.start_scope(ScopeKind::New);
        assert_eq!(ctx.scope_index().as_u32(), 3);
        nest_scope.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);
    }

    #[test]
    fn loop_scope() {
        let mut ctx = Context::new();
        assert!(!ctx.is_in_loop());

        let loop1 = ctx.start_scope(ScopeKind::Loop);
        assert_eq!(ctx.scope_index().as_u32(), 2);
        assert!(ctx.is_in_loop());

        let no_loop = ctx.start_scope(ScopeKind::New);
        assert_eq!(ctx.scope_index().as_u32(), 3);
        assert!(!ctx.is_in_loop());

        no_loop.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 2);
        assert!(ctx.is_in_loop());

        loop1.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);
        assert!(!ctx.is_in_loop());
    }

    #[test]
    fn loop_scope2() {
        let mut ctx = Context::new();
        assert!(!ctx.is_in_loop());

        let loop_scope = ctx.start_scope(ScopeKind::Loop);
        assert_eq!(ctx.scope_index().as_u32(), 2);
        assert!(ctx.is_in_loop());
        loop_scope.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);

        let loop_scope = ctx.start_scope(ScopeKind::Loop);
        assert_eq!(ctx.scope_index().as_u32(), 3);
        assert!(ctx.is_in_loop());
        loop_scope.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);
    }

    #[test]
    fn nested_loop_scope() {
        let mut ctx = Context::new();
        assert!(!ctx.is_in_loop());

        let loop1 = ctx.start_scope(ScopeKind::Loop);
        assert_eq!(ctx.scope_index().as_u32(), 2);
        assert!(ctx.is_in_loop());

        let loop2 = ctx.start_scope(ScopeKind::Loop);
        assert_eq!(ctx.scope_index().as_u32(), 3);
        assert!(ctx.is_in_loop());

        loop2.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 2);
        assert!(ctx.is_in_loop());

        loop1.finish(&mut ctx);
        assert_eq!(ctx.scope_index().as_u32(), 1);
        assert!(!ctx.is_in_loop());
    }
}
