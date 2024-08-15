use super::*;
use core::{cell::RefCell, mem::forget};
use foundation::{il::LocalId, ir::Strage};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct Context<'s> {
    block_vars_count: internal::NestedCounter,
    loop_vars_count: internal::NestedCounter,
    id_generator: internal::LocalIdGenerator<'s>,
    func_list: Rc<RefCell<Vec<Fragment>>>,
    pub(crate) strage: &'s Strage,
    pub(crate) capture_db: &'s database::FunctionCapture,
}

impl<'s> Context<'s> {
    pub(crate) fn new(strage: &'s Strage, capture_db: &'s database::FunctionCapture) -> Self {
        Self {
            block_vars_count: internal::NestedCounter::new(),
            loop_vars_count: internal::NestedCounter::new(),
            id_generator: internal::LocalIdGenerator::new(),
            func_list: Rc::new(RefCell::new(Vec::new())),
            strage,
            capture_db,
        }
    }

    pub(crate) fn new_with(ctx: &mut Self) -> Self {
        Self {
            block_vars_count: internal::NestedCounter::new(),
            loop_vars_count: internal::NestedCounter::new(),
            id_generator: internal::LocalIdGenerator::new(),
            func_list: Rc::clone(&ctx.func_list),
            strage: ctx.strage,
            capture_db: ctx.capture_db,
        }
    }

    pub(crate) fn start_block(&mut self) -> BlockMarker {
        self.block_vars_count.start_section();
        BlockMarker
    }

    pub(crate) fn start_loop(&mut self) -> LoopMarker {
        self.loop_vars_count.start_section();
        LoopMarker
    }

    pub(crate) fn get_loop_local_count(&self) -> usize {
        self.loop_vars_count
            .get_current_count()
            .expect("[BUG] This should be called after `Context::start_loop()` is called.")
    }

    pub(crate) fn get_block_local_count(&self) -> usize {
        self.block_vars_count
            .get_current_count()
            .expect("[BUG] This should be called after `Context::begin_block()` is called.")
    }

    pub(crate) fn add_local(&mut self, name: &'s str) -> LocalId {
        self.block_vars_count.increment(1);
        self.loop_vars_count.increment(1);
        self.id_generator.add_local(name)
    }

    pub(crate) fn add_function(&mut self, fragment: Fragment) -> FunctionListId {
        let len = self.func_list.borrow().len();
        self.func_list.borrow_mut().push(fragment);
        FunctionListId(len)
    }

    pub(crate) fn drop_local(&mut self, count: usize) {
        self.id_generator.drop_local(count);
        self.block_vars_count.decrement(count);
        self.loop_vars_count.decrement(count);
    }

    pub(crate) fn resolve_local(&self, name: &'s str) -> LocalId {
        self.id_generator.resolve_local(name).unwrap_or_else(|| {
            panic!("All undefined local variables error should be caught by upper layer. (undefined: {name})")
        })
    }

    pub(crate) fn finish_with(self, fragment: Fragment) -> (Vec<il::ICode>, ()) {
        let (all_code_source, funcid2index) = {
            let func_list = Rc::try_unwrap(self.func_list)
                .expect("[BUG] Context::finish_with() should be called in the outermost Context.")
                .into_inner();
            let mut codes = Vec::new();
            let mut id2idx = Vec::new();
            codes.extend(fragment.finish());
            for func in func_list {
                id2idx.push(codes.len());
                codes.extend(func.finish());
            }
            (codes, id2idx)
        };
        let mut codes = Vec::with_capacity(all_code_source.len());
        for code in all_code_source {
            use il::ICode::*;
            use ICodeSource as Src;
            #[rustfmt::skip]
            let code = match code {
                Src::LoadIntObject(x)       => LoadIntObject(x),
                Src::LoadFloatObject(x)     => LoadFloatObject(x),
                Src::LoadStringObject(x)    => LoadStringObject(x),
                Src::LoadBoolObject(x)      => LoadBoolObject(x),
                Src::LoadNilObject          => LoadNilObject,
                Src::LoadLocal(x)           => LoadLocal(x),
                Src::Unload                 => Unload,
                Src::StoreLocal(x)          => StoreLocal(x),
                Src::StoreNewLocal          => StoreNewLocal,
                Src::MakeArray(x)           => MakeArray(x),
                Src::MakeTable(x, _)        => MakeTable(x),
                Src::DropLocal(x)           => DropLocal(x),
                Src::Jump(x)                => Jump(x),
                Src::JumpIfTrue(x)          => JumpIfTrue(x),
                Src::JumpIfFalse(x)         => JumpIfFalse(x),
                Src::Call(x, _)             => Call(x),
                Src::CallMethod(x, y, _)    => CallMethod(x, y),
                Src::SetItem(_)             => SetItem,
                Src::GetItem(_)             => GetItem,
                Src::SetMethod(x, _)        => SetMethod(x),
                Src::Add(_)                 => Add,
                Src::Sub(_)                 => Sub,
                Src::Mul(_)                 => Mul,
                Src::Div(_)                 => Div,
                Src::Mod(_)                 => Mod,
                Src::Unm(_)                 => Unm,
                Src::Unp(_)                 => Unp,
                Src::Not(_)                 => Not,
                Src::Eq(_)                  => Eq,
                Src::NotEq(_)               => NotEq,
                Src::Less(_)                => Less,
                Src::LessEq(_)              => LessEq,
                Src::Greater(_)             => Greater,
                Src::GreaterEq(_)           => GreaterEq,
                Src::Concat(_)              => Concat,
                Src::BitAnd(_)              => BitAnd,
                Src::BitOr(_)               => BitOr,
                Src::BitXor(_)              => BitXor,
                Src::BitNot(_)              => BitNot,
                Src::ShiftL(_)              => ShiftL,
                Src::ShiftR(_)              => ShiftR,
                Src::GetIter                => GetIter,
                Src::IterMoveNext           => IterMoveNext,
                Src::IterCurrent            => IterCurrent,
                Src::BeginFuncSection       => BeginFuncSection,
                Src::FuncSetProperty(x, id) => FuncSetProperty(x, funcid2index[id.0]),
                Src::FuncAddCapture(x)      => FuncAddCapture(x),
                Src::EndFuncSection         => EndFuncSection,
                Src::Leave                  => Leave,
                Src::Tombstone              => panic!("[BUG] Tombstone should not be in the final code."),
            };
            codes.push(code);
        }
        (codes, ())
    }
}

#[must_use]
pub(crate) struct BlockMarker;
impl BlockMarker {
    pub(crate) fn finish(self, ctx: &mut Context<'_>) {
        forget(self);
        let block_cnt = ctx.block_vars_count.end_section();
        ctx.id_generator.drop_local(block_cnt);
        ctx.loop_vars_count.decrement(block_cnt);
    }
}
impl Drop for BlockMarker {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("BlockMarker must be completed with finish() method");
        }
    }
}

#[must_use]
pub(crate) struct LoopMarker;
impl LoopMarker {
    pub(crate) fn finish(self, ctx: &mut Context<'_>) {
        forget(self);
        ctx.loop_vars_count.end_section();
    }
}
impl Drop for LoopMarker {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("LoopMarker must be completed with finish() method");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FunctionListId(usize);

mod internal {
    use super::*;
    use rustc_hash::FxHashMap;

    #[derive(Debug)]
    pub struct NestedCounter {
        stack: Vec<usize>,
    }

    impl NestedCounter {
        pub const fn new() -> Self {
            Self { stack: Vec::new() }
        }

        pub fn start_section(&mut self) {
            self.stack.push(0);
        }

        // TODO: use dorp marker
        pub fn end_section(&mut self) -> usize {
            self.stack.pop().expect(
                "[BUG] this should be called after `NestedCounter::start_new_section()` is called.",
            )
        }

        pub fn increment(&mut self, count: usize) {
            if let Some(last) = self.stack.last_mut() {
                *last += count;
            }
        }

        pub fn decrement(&mut self, count: usize) {
            if let Some(last) = self.stack.last_mut() {
                *last -= count;
            }
        }

        pub fn get_current_count(&self) -> Option<usize> {
            self.stack.last().copied()
        }
    }

    #[derive(Debug)]
    pub struct LocalIdGenerator<'s> {
        map: FxHashMap<&'s str, LocalId>,
        vec: Vec<(&'s str, LocalId)>,
    }

    impl<'s> LocalIdGenerator<'s> {
        pub fn new() -> Self {
            Self {
                map: FxHashMap::default(),
                vec: Vec::new(),
            }
        }

        pub fn add_local(&mut self, name: &'s str) -> LocalId {
            let id = LocalId::new(self.vec.len());
            let old_id = self.map.insert(name, id);
            let restore = if let Some(old_id) = old_id {
                (name, old_id)
            } else {
                (name, id)
            };
            self.vec.push(restore);
            id
        }

        pub fn resolve_local(&self, name: &'s str) -> Option<LocalId> {
            self.map.get(name).copied()
        }

        pub fn drop_local(&mut self, count: usize) {
            for _ in 0..count {
                let (name, id) = self.vec.pop().expect(
                    "[BUG] `count` should be less than or equal to the number of variables.",
                );
                let mut stored_entry = match self.map.entry(name) {
                    std::collections::hash_map::Entry::Occupied(x) => x,
                    std::collections::hash_map::Entry::Vacant(_) => {
                        unreachable!("This is ensured by `LocalIdGenerator::add_local()`.")
                    }
                };
                if stored_entry.get() != &id {
                    stored_entry.insert(id);
                } else {
                    stored_entry.remove();
                }
            }
        }
    }
}
