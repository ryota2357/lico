use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

impl VariableId {
    #[inline]
    pub const fn new_manual(id: usize) -> Self {
        Self(id)
    }
}

impl Deref for VariableId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Context<'src> {
    block_vars_count: internal::NestedCounter,
    loop_vars_count: internal::NestedCounter,
    id_generator: internal::VariableIdGenerator<'src>,
}

impl<'src> Context<'src> {
    pub fn new() -> Self {
        Self {
            block_vars_count: internal::NestedCounter::new(),
            loop_vars_count: internal::NestedCounter::new(),
            id_generator: internal::VariableIdGenerator::new(),
        }
    }

    pub fn begin_block(&mut self) {
        self.block_vars_count.start_section();
    }

    pub fn end_block(&mut self) {
        let block_cnt = self.block_vars_count.end_section();
        self.loop_vars_count.decrement(block_cnt);
    }

    pub fn begin_loop(&mut self) {
        self.loop_vars_count.start_section();
    }

    pub fn end_loop(&mut self) {
        self.loop_vars_count.end_section();
    }

    /// Returns the number of locals in the current loop section.
    /// Returns [`None`] if there is no current loop section.
    #[inline]
    pub fn get_loop_vars_count(&self) -> Option<usize> {
        self.loop_vars_count.get_current_count()
    }

    #[inline]
    pub fn get_block_local_count(&self) -> usize {
        self.block_vars_count
            .get_current_count()
            .expect("[BUG] This should be called after `Context::begin_block()` is called.")
    }

    #[inline]
    pub fn add_variable(&mut self, name: &'src str) -> VariableId {
        self.block_vars_count.increment(1);
        self.loop_vars_count.increment(1);
        self.id_generator.add_variable(name)
    }

    pub fn add_variable_many(&mut self, names: impl IntoIterator<Item = &'src str>) {
        for name in names.into_iter() {
            self.add_variable(name);
        }
    }

    #[inline]
    pub fn drop_variable(&mut self, count: usize) {
        self.id_generator.drop_variable(count);
        self.block_vars_count.decrement(count);
        self.loop_vars_count.decrement(count);
    }

    #[inline]
    pub fn resolve_variable(&self, name: &'src str) -> Option<VariableId> {
        self.id_generator.resolve_variable(name)
    }
}

mod internal {
    use super::*;
    use rustc_hash::FxHashMap;

    #[derive(Debug)]
    pub struct NestedCounter {
        stack: Vec<usize>,
    }

    impl NestedCounter {
        #[inline]
        pub const fn new() -> Self {
            Self { stack: Vec::new() }
        }

        #[inline]
        pub fn start_section(&mut self) {
            self.stack.push(0);
        }

        pub fn end_section(&mut self) -> usize {
            self.stack.pop().expect(
                "[BUG] this should be called after `NestedCounter::start_new_section()` is called.",
            )
        }

        #[inline]
        pub fn increment(&mut self, count: usize) {
            if let Some(last) = self.stack.last_mut() {
                *last += count;
            }
        }

        #[inline]
        pub fn decrement(&mut self, count: usize) {
            if let Some(last) = self.stack.last_mut() {
                *last -= count;
            }
        }

        #[inline]
        pub fn get_current_count(&self) -> Option<usize> {
            self.stack.last().copied()
        }
    }

    #[derive(Debug)]
    pub struct VariableIdGenerator<'src> {
        map: FxHashMap<&'src str, VariableId>,
        vec: Vec<(&'src str, VariableId)>,
    }

    impl<'src> VariableIdGenerator<'src> {
        #[inline]
        pub fn new() -> Self {
            Self {
                map: FxHashMap::default(),
                vec: Vec::new(),
            }
        }

        pub fn add_variable(&mut self, name: &'src str) -> VariableId {
            let id = VariableId(self.vec.len());
            let old_id = self.map.insert(name, id);
            let restore = if let Some(old_id) = old_id {
                (name, old_id)
            } else {
                (name, id)
            };
            self.vec.push(restore);
            id
        }

        #[inline]
        pub fn resolve_variable(&self, name: &'src str) -> Option<VariableId> {
            self.map.get(name).copied()
        }

        pub fn drop_variable(&mut self, count: usize) {
            for _ in 0..count {
                let (name, id) = self.vec.pop().expect(
                    "[BUG] `count` should be less than or equal to the number of variables.",
                );
                let mut stored_entry = match self.map.entry(name) {
                    std::collections::hash_map::Entry::Occupied(x) => x,
                    std::collections::hash_map::Entry::Vacant(_) => {
                        unreachable!("This is ensured by `VariableIdGenerator::new_variable()`.")
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
