use super::*;
use crate::{
    collections::arena::{Arena, Index, Slice},
    syntax::{SyntaxNode, SyntaxToken},
};
use compact_str::CompactString;
use core::fmt;

enum RawData {
    Value(SyntaxNode, Value),
    Effect(SyntaxNode, Effect),
    String(SyntaxToken, CompactString),
    Symbol(SyntaxToken, Symbol),
}

pub struct Strage {
    arena: Arena<RawData>,
}

impl<'s> Strage {
    pub fn get<K: StrageKey<'s>>(&'s self, key: &K) -> K::ValueRef {
        key.get(self)
    }
}

impl fmt::Debug for Strage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Strage")
            .field("len", &self.arena.len())
            .field(
                "data",
                &self
                    .arena
                    .iter()
                    .map(|(i, d)| {
                        let data = match d {
                            RawData::Value(ptr, value) => {
                                format!("Value({:?}, {:?})", ptr.text_range(), value)
                            }
                            RawData::Effect(ptr, effect) => {
                                format!("Effect({:?}, {:?})", ptr.text_range(), effect)
                            }
                            RawData::String(token, string) => {
                                format!("String({:?}, {:?})", token.text(), string)
                            }
                            RawData::Symbol(token, symbol) => {
                                format!("Symbol({:?}, {:?})", token.text(), symbol)
                            }
                        };
                        (i, data)
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

pub trait StrageKey<'s> {
    type ValueRef: 's;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueKey(Option<Index<RawData>>);

impl ValueKey {
    pub fn as_u32(&self) -> u32 {
        self.0.map_or(0, |index| index.as_u32())
    }
}

impl<'s> StrageKey<'s> for ValueKey {
    type ValueRef = Option<(&'s SyntaxNode, &'s Value)>;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get(self.0?);
        match data {
            RawData::Value(ptr, value) => Some((ptr, value)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueSliceKey(Slice<RawData>);

impl ValueSliceKey {
    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'s> StrageKey<'s> for ValueSliceKey {
    type ValueRef = impl ExactSizeIterator<Item = (&'s SyntaxNode, &'s Value)>;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get_slice(self.0);
        data.iter().map(|data| match data {
            RawData::Value(ptr, value) => (ptr, value),
            _ => unreachable!(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectsKey(Slice<RawData>);

impl EffectsKey {
    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'s> StrageKey<'s> for EffectsKey {
    type ValueRef = impl ExactSizeIterator<Item = (&'s SyntaxNode, &'s Effect)>;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get_slice(self.0);
        data.iter().map(|data| match data {
            RawData::Effect(ptr, effect) => (ptr, effect),
            _ => unreachable!(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringKey(Option<Index<RawData>>);

impl StringKey {
    pub fn as_u64(&self) -> u64 {
        self.0.map_or(0, |index| index.as_u32() as u64)
    }
}

impl<'s> StrageKey<'s> for StringKey {
    type ValueRef = Option<(&'s SyntaxToken, &'s CompactString)>;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get(self.0?);
        match data {
            RawData::String(token, string) => Some((token, string)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringSliceKey(Slice<RawData>);

impl StringSliceKey {
    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'s> StrageKey<'s> for StringSliceKey {
    type ValueRef = impl ExactSizeIterator<Item = (&'s SyntaxToken, &'s CompactString)>;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get_slice(self.0);
        data.iter().map(|data| match data {
            RawData::String(token, string) => (token, string),
            _ => unreachable!(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolKey(Option<Index<RawData>>);

impl SymbolKey {
    pub fn as_u32(&self) -> u32 {
        self.0.map_or(0, |index| index.as_u32())
    }
}

impl<'s> StrageKey<'s> for SymbolKey {
    type ValueRef = Option<(&'s SyntaxToken, &'s Symbol)>;
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get(self.0?);
        match data {
            RawData::Symbol(token, symbol) => Some((token, symbol)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionKey(Slice<RawData>);

impl FunctionKey {
    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }
}

impl<'s> StrageKey<'s> for FunctionKey {
    type ValueRef = (
        impl Iterator<Item = (&'s SyntaxToken, &'s Symbol)>,
        impl Iterator<Item = (&'s SyntaxNode, &'s Effect)>,
    );
    fn get(&self, strage: &'s Strage) -> Self::ValueRef {
        let data = strage.arena.get_slice(self.0);
        let symbols = data.iter().filter_map(|data| match data {
            RawData::Symbol(token, symbol) => Some((token, symbol)),
            _ => None,
        });
        let effects = data.iter().filter_map(|data| match data {
            RawData::Effect(ptr, effect) => Some((ptr, effect)),
            _ => None,
        });
        (symbols, effects)
    }
}

pub struct StrageBuilder {
    arena: Arena<RawData>,
}

// ここの add_* 系は StrageKey に type Key を追加して、StrageKey::gen_key() を使うようにして、add() で追加するようにしたい。
// けど、Key = impl Iterator<Item = (SyntaxNode, Value)> が次のエラーになる。
//   unconstrained opaque type `Value` must be used in combination with a concrete type within the same impl
// なお、StrageKey::gen_key() は
//   fn gen_key(value: Self::Value, strage: &'s mut Strage) -> Self;
// というシグネチャ
impl StrageBuilder {
    pub const fn new() -> Self {
        StrageBuilder {
            arena: Arena::new(),
        }
    }

    pub fn finish(self) -> Strage {
        Strage { arena: self.arena }
    }

    pub fn add_value(&mut self, value: Option<(SyntaxNode, Value)>) -> ValueKey {
        match value {
            Some((ptr, value)) => {
                let data = RawData::Value(ptr, value);
                let idx = self.arena.alloc(data);
                ValueKey(Some(idx))
            }
            None => ValueKey(None),
        }
    }

    pub fn add_value_many(
        &mut self,
        values: impl Iterator<Item = (SyntaxNode, Value)>,
    ) -> ValueSliceKey {
        let iter = values.map(|(ptr, value)| RawData::Value(ptr, value));
        let slice = self.arena.alloc_many(iter);
        ValueSliceKey(slice)
    }

    pub fn add_effects(
        &mut self,
        effects: impl Iterator<Item = (SyntaxNode, Effect)>,
    ) -> EffectsKey {
        let iter = effects.map(|(ptr, effect)| RawData::Effect(ptr, effect));
        let slice = self.arena.alloc_many(iter);
        EffectsKey(slice)
    }

    pub fn add_string(&mut self, string: Option<(SyntaxToken, CompactString)>) -> StringKey {
        match string {
            Some((node, string)) => {
                let data = RawData::String(node, string);
                let idx = self.arena.alloc(data);
                StringKey(Some(idx))
            }
            None => StringKey(None),
        }
    }

    pub fn add_string_many(
        &mut self,
        strings: impl Iterator<Item = (SyntaxToken, CompactString)>,
    ) -> StringSliceKey {
        let iter = strings.map(|(token, string)| RawData::String(token, string));
        let slice = self.arena.alloc_many(iter);
        StringSliceKey(slice)
    }

    pub fn add_symbol(&mut self, symbol: Option<(SyntaxToken, Symbol)>) -> SymbolKey {
        match symbol {
            Some((node, symbol)) => {
                let data = RawData::Symbol(node, symbol);
                let idx = self.arena.alloc(data);
                SymbolKey(Some(idx))
            }
            None => SymbolKey(None),
        }
    }

    pub fn add_function(
        &mut self,
        symbols: impl Iterator<Item = (SyntaxToken, Symbol)>,
        effects: impl Iterator<Item = (SyntaxNode, Effect)>,
    ) -> FunctionKey {
        let symbols_iter = symbols.map(|(ptr, symbol)| RawData::Symbol(ptr, symbol));
        let effects_iter = effects.map(|(ptr, effect)| RawData::Effect(ptr, effect));
        let slice = self.arena.alloc_many(symbols_iter.chain(effects_iter));
        FunctionKey(slice)
    }
}

impl Default for StrageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
