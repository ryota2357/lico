use compact_str::CompactString;
use core::{
    cell::RefCell,
    fmt,
    mem::{replace, take},
};
use foundation::ir::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;

pub struct FunctionCapture {
    pub(crate) map: FxHashMap<FunctionCaptureKey, FxHashSet<CompactString>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionCaptureKey(u64);

impl From<&FunctionKey> for FunctionCaptureKey {
    fn from(value: &FunctionKey) -> Self {
        const {
            // Simple check that FunctionKey is NonZero type.
            assert!(size_of::<FunctionKey>() == size_of::<Option<FunctionKey>>());
        };
        // Of course, the above check is not perfect, So I check it at runtime.
        assert!(value.as_u64() != 0);
        FunctionCaptureKey(value.as_u64())
    }
}

impl From<&Module> for FunctionCaptureKey {
    fn from(_value: &Module) -> Self {
        // FunctionCaptureKey is only from `FunctionKey` and `Module`.
        // `FunctionKey` is NonZero type (I checked it above assetions), so I can use 0 as a key
        // for `Module`.
        FunctionCaptureKey(0)
    }
}

pub enum CaptureHashSetRef<'s> {
    Empty,
    Occupied(&'s FxHashSet<CompactString>),
}

impl FunctionCapture {
    pub fn build_with(
        module: &Module,
        defaults: impl IntoIterator<Item = &'static str>,
    ) -> (Self, Vec<&'static str>) {
        walk(module, defaults.into_iter().collect())
    }

    pub fn get_capture(&self, key: impl Into<FunctionCaptureKey>) -> CaptureHashSetRef {
        match self.map.get(&key.into()) {
            Some(set) => CaptureHashSetRef::Occupied(set),
            None => CaptureHashSetRef::Empty,
        }
    }

    pub fn iter_captures(
        &self,
    ) -> impl Iterator<Item = (&FunctionCaptureKey, &FxHashSet<CompactString>)> {
        self.map.iter()
    }
}

impl fmt::Debug for FunctionCapture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.map.iter()).finish()
    }
}

impl<'s> CaptureHashSetRef<'s> {
    pub fn contains(&self, name: &str) -> bool {
        match &self {
            CaptureHashSetRef::Empty => false,
            CaptureHashSetRef::Occupied(hash) => hash.contains(name),
        }
    }

    pub fn iter(&self) -> CaptureHashSetRefIter<'s> {
        match &self {
            CaptureHashSetRef::Empty => CaptureHashSetRefIter::Empty,
            CaptureHashSetRef::Occupied(hash) => CaptureHashSetRefIter::Occupied(hash.iter()),
        }
    }
}

pub enum CaptureHashSetRefIter<'s> {
    Empty,
    Occupied(std::collections::hash_set::Iter<'s, CompactString>),
}

impl<'s> Iterator for CaptureHashSetRefIter<'s> {
    type Item = &'s CompactString;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CaptureHashSetRefIter::Empty => None,
            CaptureHashSetRefIter::Occupied(iter) => iter.next(),
        }
    }
}

impl ExactSizeIterator for CaptureHashSetRefIter<'_> {
    fn len(&self) -> usize {
        match self {
            CaptureHashSetRefIter::Empty => 0,
            CaptureHashSetRefIter::Occupied(iter) => iter.len(),
        }
    }
}

use internal::*;

impl<'strage> Walkable<'strage> for EffectsKey {
    fn accept(&self, w: &mut Walker<'strage>) {
        for (_, effect) in w.strage.get(self) {
            w.go(effect);
        }
    }
}

impl<'strage> Walkable<'strage> for Effect {
    fn accept(&self, w: &mut Walker<'strage>) {
        use Effect::*;
        match self {
            MakeLocal { name, value } => {
                w.go(value);
                w.insert_def(name);
            }
            MakeFunc { name, func } => {
                w.insert_def(name);
                w.go_function(*func, |w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                });
            }
            SetLocal { local, value } => {
                w.use_local(local);
                w.go(value);
            }
            SetIndex {
                target,
                index,
                value,
            } => {
                w.go(value);
                w.go(index);
                w.go(target);
            }
            SetField {
                target,
                field: _,
                value,
            } => {
                w.go(value);
                w.go(target);
            }
            SetFieldFunc {
                table,
                path: _,
                func,
            } => {
                w.go_function(*func, |w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                });
                w.use_local(table);
            }
            SetMethod {
                table,
                path: _,
                name: _,
                func,
            } => {
                w.go_function(*func, |w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                });
                w.use_local(table);
            }
            Branch {
                condition,
                then,
                else_,
            } => {
                w.go(condition);
                w.go_branch(|w| w.go(then));
                w.go_branch(|w| w.go(else_));
            }
            LoopFor {
                variable,
                iterable,
                effects,
            } => {
                w.go(iterable);
                w.go_branch(|w| {
                    w.insert_def(variable);
                    w.go(effects);
                });
            }
            LoopWhile { condition, effects } => {
                w.go(condition);
                w.go_branch(|w| w.go(effects));
            }
            Scope { body } => {
                w.go_branch(|w| w.go(body));
            }
            Call { value, args } => {
                w.go(value);
                for (_, arg) in w.strage.get(args) {
                    w.go(arg);
                }
            }
            MethodCall {
                table,
                name: _,
                args,
            } => {
                w.go(table);
                for (_, arg) in w.strage.get(args) {
                    w.go(arg);
                }
            }
            Return { value } => {
                w.go(value);
            }
            BreakLoop => {}
            ContinueLoop => {}
            NoEffectValue { value } => {
                w.go(value);
            }
        }
    }
}

impl<'strage> Walkable<'strage> for ValueKey {
    fn accept(&self, walker: &mut Walker<'strage>) {
        let Some((_, value)) = walker.strage.get(self) else {
            return;
        };
        walker.go(value);
    }
}

impl<'strage> Walkable<'strage> for Value {
    fn accept(&self, w: &mut Walker<'strage>) {
        use Value::*;
        match self {
            Branch {
                condition,
                then,
                then_tail,
                else_,
                else_tail,
            } => {
                w.go(condition);
                w.go_branch(|w| {
                    w.go(then);
                    w.go(then_tail);
                });
                w.go_branch(|w| {
                    w.go(else_);
                    w.go(else_tail);
                });
            }
            Prefix { op: _, value } => {
                w.go(value);
            }
            Binary { op: _, lhs, rhs } => {
                w.go(lhs);
                w.go(rhs);
            }
            Call { value, args } => {
                w.go(value);
                for (_, arg) in w.strage.get(args) {
                    w.go(arg);
                }
            }
            Index { value, index } => {
                w.go(value);
                w.go(index);
            }
            Field { value, name: _ } => {
                w.go(value);
            }
            MethodCall {
                value,
                name: _,
                args,
            } => {
                w.go(value);
                for (_, arg) in w.strage.get(args) {
                    w.go(arg);
                }
            }
            Block { effects, tail } => {
                w.go_branch(|w| {
                    w.go(effects);
                    w.go(tail);
                });
            }
            Local { name } => {
                w.use_local(name);
            }
            Int(_) | Float(_) | String(_) | Bool(_) | Nil => {}
            Function(func) => {
                w.go_function(*func, |w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                });
            }
            Array { elements } => {
                for (_, element) in w.strage.get(elements) {
                    w.go(element);
                }
            }
            Table { fields } => {
                for (key, value) in fields.iter() {
                    match key {
                        TableKeyName::Value(key) => {
                            w.go(key);
                        }
                        TableKeyName::String(_) => {}
                    }
                    w.go(value);
                }
            }
        }
    }
}

mod internal {
    use super::*;

    pub struct Walker<'strage> {
        pub strage: &'strage Strage,
        db: Rc<RefCell<FunctionCapture>>,
        master: Vec<(FunctionKey, FxHashMap<&'strage str, u32>)>,
        defs: FxHashMap<&'strage str, u32>,
        current: FunctionCaptureKey,
        defs_rev: Vec<&'strage str>,

        // From the concept of Lico Language, the number of default functions/variables is small,
        // so I use Vec instead of HashSet.
        unused_defaunts: Vec<&'static str>,
    }

    pub(super) trait Walkable<'strage> {
        fn accept(&self, w: &mut Walker<'strage>);
    }

    pub(crate) fn walk(
        module: &Module,
        defaults: Vec<&'static str>,
    ) -> (FunctionCapture, Vec<&'static str>) {
        let db = Rc::new(RefCell::new(FunctionCapture {
            map: FxHashMap::default(),
        }));
        let mut walker = Walker {
            strage: module.strage(),
            db,
            master: Vec::new(),
            defs: FxHashMap::default(),
            current: FunctionCaptureKey::from(module),
            defs_rev: Vec::new(),
            unused_defaunts: defaults.clone(),
        };
        walker.go(module.effects());
        assert!(walker.master.is_empty());
        let db = Rc::try_unwrap(walker.db).unwrap().into_inner();
        let used_defaults = defaults
            .into_iter()
            .filter(|s| !walker.unused_defaunts.contains(s))
            .collect();
        (db, used_defaults)
    }

    impl<'strage> Walker<'strage> {
        pub(super) fn go(&mut self, key: &impl Walkable<'strage>) {
            key.accept(self);
        }

        pub(super) fn go_branch(&mut self, f: impl FnOnce(&mut Walker<'strage>)) {
            let save_defs_rev = take(&mut self.defs_rev);
            f(self);
            let remove_def = replace(&mut self.defs_rev, save_defs_rev);
            for def in remove_def {
                *self.defs.get_mut(def).unwrap() -= 1;
                if *self.defs.get(def).unwrap() == 0 {
                    self.defs.remove(def);
                }
            }
        }

        pub(super) fn go_function(
            &mut self,
            func_key: FunctionKey,
            f: impl FnOnce(&mut Walker<'strage>),
        ) {
            let save_current = self.current;

            let mut master = take(&mut self.master);
            master.push((func_key, take(&mut self.defs)));
            let mut walker = Walker {
                strage: self.strage,
                db: Rc::clone(&self.db),
                master,
                defs: FxHashMap::default(),
                current: FunctionCaptureKey::from(&func_key),
                defs_rev: Vec::new(),
                unused_defaunts: take(&mut self.unused_defaunts),
            };
            f(&mut walker);
            walker.master.pop();

            self.master = walker.master;
            self.defs = walker.defs;
            self.current = save_current;
            self.defs_rev = walker.defs_rev;
            self.unused_defaunts = walker.unused_defaunts;
        }

        #[allow(private_bounds)]
        pub(super) fn insert_def(&mut self, symbol: impl IntoSymbol<'strage>) {
            let Some(symbol) = symbol.into_symbol(self.strage) else {
                return;
            };
            *self.defs.entry(symbol.text()).or_insert(0) += 1;
        }

        pub(super) fn use_local(&mut self, symbol: &SymbolKey) {
            let Some((_, symbol)) = self.strage.get(symbol) else {
                return;
            };
            let symbol_str = symbol.text();
            if self.defs.contains_key(symbol_str) {
                return;
            }

            // TODO: remove aftter full build type check
            fn check_double_embedded<T: core::iter::DoubleEndedIterator>(_x: T) {}
            check_double_embedded(self.defs_rev.iter());

            let mut found_index = -1;
            for (i, (_, defs)) in self.master.iter().enumerate().rev() {
                if defs.contains_key(symbol_str) {
                    found_index = i as isize;
                    break;
                }
            }
            if found_index != -1 {
                for (func, defs) in &mut self.master[(found_index + 1) as usize..] {
                    debug_assert!(!defs.contains_key(symbol_str));
                    defs.insert(symbol_str, 1);
                    debug_assert!(!self.db.borrow().get_capture(&*func).contains(symbol_str));
                    self.db
                        .borrow_mut()
                        .map
                        .entry((&*func).into())
                        .or_default()
                        .insert(CompactString::from(symbol_str));
                }
            } else if self.unused_defaunts.contains(&symbol_str) {
                self.unused_defaunts.retain(|s| *s != symbol_str);
            } else {
                todo!(
                    "Implement undefined local variable error. (undefined: {})",
                    symbol_str
                );
            }
        }
    }

    trait IntoSymbol<'s> {
        fn into_symbol(self, strage: &'s Strage) -> Option<&'s Symbol>;
    }
    impl<'s> IntoSymbol<'s> for &SymbolKey {
        fn into_symbol(self, strage: &'s Strage) -> Option<&'s Symbol> {
            strage.get(self).map(|(_, symbol)| symbol)
        }
    }
    impl<'s> IntoSymbol<'s> for &'s Symbol {
        #[inline(always)]
        fn into_symbol(self, _strage: &'s Strage) -> Option<&'s Symbol> {
            Some(self)
        }
    }
}
