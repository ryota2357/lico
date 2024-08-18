use compact_str::CompactString;
use core::{cell::RefCell, fmt, mem::take};
use foundation::ir::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;

pub struct FunctionCapture {
    map: FxHashMap<u64, FxHashSet<CompactString>>,
}

pub trait FunctionCaptureKey {
    fn as_db_key(&self) -> u64;
}

impl FunctionCaptureKey for &SymbolKey {
    fn as_db_key(&self) -> u64 {
        self.as_u32() as u64
    }
}
impl FunctionCaptureKey for &FunctionKey {
    fn as_db_key(&self) -> u64 {
        self.as_u64()
    }
}
impl FunctionCaptureKey for &Module {
    fn as_db_key(&self) -> u64 {
        self.effects().as_u64()
    }
}

pub struct CaptureHashSetRef<'s>(CaptureHashSetRefInner<'s>);

enum CaptureHashSetRefInner<'s> {
    Empty,
    Occupied(&'s FxHashSet<CompactString>),
}

impl FunctionCapture {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }

    pub fn build_with(&mut self, module: &Module) {
        let db = Rc::new(RefCell::new(FunctionCapture {
            map: take(&mut self.map),
        }));
        let mut walker = Walker::new(module.strage(), Rc::clone(&db));
        walker.go(module.effects());
        self.map = take(&mut walker.take_db().map);
    }

    pub fn insert(&mut self, key: impl FunctionCaptureKey, value: impl Into<CompactString>) {
        self.map
            .entry(key.as_db_key())
            .or_default()
            .insert(value.into());
    }

    pub fn get_capture(&self, key: impl FunctionCaptureKey) -> CaptureHashSetRef {
        let inner = match self.map.get(&key.as_db_key()) {
            Some(hash) => CaptureHashSetRefInner::Occupied(hash),
            None => CaptureHashSetRefInner::Empty,
        };
        CaptureHashSetRef(inner)
    }

    pub fn iter_captures(&self) -> impl Iterator<Item = (&u64, &FxHashSet<CompactString>)> {
        self.map.iter()
    }
}

impl Default for FunctionCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for FunctionCapture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.map.iter()).finish()
    }
}

impl<'s> CaptureHashSetRef<'s> {
    pub fn contains(&self, name: &str) -> bool {
        match &self.0 {
            CaptureHashSetRefInner::Empty => false,
            CaptureHashSetRefInner::Occupied(hash) => hash.contains(name),
        }
    }

    pub fn iter(&self) -> CaptureHashSetRefIter<'s> {
        match &self.0 {
            CaptureHashSetRefInner::Empty => CaptureHashSetRefIter::Empty,
            CaptureHashSetRefInner::Occupied(hash) => CaptureHashSetRefIter::Occupied(hash.iter()),
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

impl<'walker, 'strage: 'walker> Walkable<'walker, 'strage> for EffectsKey {
    fn accept(&self, w: &mut Walker<'walker, 'strage>) {
        for (_, effect) in w.strage.get(self) {
            w.go(effect);
        }
    }
}

impl<'walker, 'strage: 'walker> Walkable<'walker, 'strage> for Effect {
    fn accept(&self, w: &mut Walker<'walker, 'strage>) {
        use Effect::*;
        match self {
            MakeLocal { name, value } => {
                w.go(value);
                w.insert_def(name);
            }
            MakeFunc { name, func } => {
                w.insert_def(name);
                w.fork(|w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                    func
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
                w.fork(|w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                    func
                });
                w.use_local(table);
            }
            SetMethod {
                table,
                path: _,
                name: _,
                func,
            } => {
                w.fork(|w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                    func
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

impl<'walker, 'strage: 'walker> Walkable<'walker, 'strage> for ValueKey {
    fn accept(&self, walker: &mut Walker<'walker, 'strage>) {
        let Some((_, value)) = walker.strage.get(self) else {
            return;
        };
        walker.go(value);
    }
}

impl<'walker, 'strage: 'walker> Walkable<'walker, 'strage> for Value {
    fn accept(&self, w: &mut Walker<'walker, 'strage>) {
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
                w.fork(|w| {
                    let (param_iter, effect_iter) = w.strage.get(func);
                    for (_, param) in param_iter {
                        w.insert_def(param);
                    }
                    for (_, effect) in effect_iter {
                        w.go(effect);
                    }
                    func
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

    #[derive(Default)]
    struct LinkedNode<'a, T> {
        value: T,
        next: Option<&'a LinkedNode<'a, T>>,
    }

    pub(super) struct Walker<'walker, 'strage: 'walker> {
        pub strage: &'strage Strage,
        db: Rc<RefCell<FunctionCapture>>,
        master_defs: LinkedNode<'walker, FxHashSet<&'strage str>>,
        defs: FxHashSet<&'strage str>,
        caps: FxHashSet<&'strage str>,
        defs_rev: Vec<&'strage str>,
    }

    pub(super) trait Walkable<'walker, 'strage: 'walker> {
        fn accept(&self, w: &mut Walker<'walker, 'strage>);
    }

    impl<'walker, 'strage: 'walker> Walker<'walker, 'strage> {
        pub(super) fn new(strage: &'strage Strage, db: Rc<RefCell<FunctionCapture>>) -> Self {
            Self {
                strage,
                db,
                master_defs: LinkedNode {
                    value: FxHashSet::default(),
                    next: None,
                },
                defs: FxHashSet::default(),
                caps: FxHashSet::default(),
                defs_rev: Vec::new(),
            }
        }

        pub(super) fn go(&mut self, key: &impl Walkable<'walker, 'strage>) {
            key.accept(self);
        }

        pub(super) fn go_branch(&mut self, f: impl FnOnce(&mut Walker<'walker, 'strage>)) {
            let save_defs_rev = take(&mut self.defs_rev);
            let mut walker = Walker {
                strage: self.strage,
                db: Rc::clone(&self.db),
                master_defs: take(&mut self.master_defs),
                defs: take(&mut self.defs),
                caps: take(&mut self.caps),
                defs_rev: Vec::new(),
            };
            f(&mut walker);
            for def in walker.defs_rev {
                walker.defs.remove(def);
            }
            self.defs_rev = save_defs_rev;
            self.master_defs = walker.master_defs;
            self.defs = walker.defs;
            self.caps = walker.caps;
        }

        pub(super) fn fork<'a, F>(&mut self, f: F)
        where
            F: FnOnce(&mut Walker<'walker, 'strage>) -> &'a FunctionKey,
        {
            let mut walker = Walker::new(self.strage, Rc::clone(&self.db));
            let key = f(&mut walker);
            let Walker {
                db: record, caps, ..
            } = walker;
            let mut record_borrow_mut = record.borrow_mut();
            for cap in caps {
                record_borrow_mut.insert(key, cap);
            }
        }

        #[allow(private_bounds)]
        pub(super) fn insert_def(&mut self, symbol: impl IntoSymbol<'strage>) {
            let Some(symbol) = symbol.into_symbol(self.strage) else {
                return;
            };
            self.defs.insert(symbol.text());
        }

        pub(super) fn use_local(&self, symbol: &SymbolKey) {
            let Some((_, symbol)) = self.strage.get(symbol) else {
                return;
            };
            let symbol_str = symbol.text();
            if self.defs.contains(symbol_str) {
                return;
            }
            let mut next = self.master_defs.next;
            while let Some(defs) = next {
                if defs.value.contains(symbol_str) {
                    return;
                }
                next = defs.next;
            }
            todo!(
                "Implement undefined local variable error. (undefined: {})",
                symbol_str
            );
        }

        pub(super) fn take_db(self) -> FunctionCapture {
            assert!(self.master_defs.next.is_none());
            Rc::try_unwrap(self.db).unwrap().into_inner()
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
