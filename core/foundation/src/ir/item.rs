use super::*;
use crate::{object::UString, syntax::SyntaxToken};
use core::num::NonZero;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeIndex(NonZero<u32>);

impl ScopeIndex {
    pub const fn new() -> Self {
        Self(unsafe { NonZero::new_unchecked(1) })
    }

    pub const fn make_next(&self) -> Self {
        let next = self.0.get() + 1;
        Self(unsafe { NonZero::new_unchecked(next) })
    }

    pub fn as_u32(&self) -> u32 {
        self.0.get()
    }
}

impl Default for ScopeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    text: CompactString,
    scope: ScopeIndex,
}

impl Symbol {
    pub fn new(text: CompactString, scope: ScopeIndex) -> Self {
        Self { text, scope }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn scope(&self) -> ScopeIndex {
        self.scope
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    MakeLocal {
        name: SymbolKey,
        value: ValueKey,
    },
    MakeFunc {
        name: SymbolKey,
        func: FunctionKey,
    },
    SetLocal {
        local: SymbolKey,
        value: ValueKey,
    },
    SetIndex {
        target: ValueKey,
        index: ValueKey,
        value: ValueKey,
    },
    SetField {
        target: ValueKey,
        field: StringKey,
        value: ValueKey,
    },
    SetFieldFunc {
        table: SymbolKey,
        path: StringSliceKey,
        func: FunctionKey,
    },
    SetMethod {
        table: SymbolKey,
        path: StringSliceKey,
        name: StringKey,
        func: FunctionKey,
    },
    Branch {
        condition: ValueKey,
        then: EffectsKey,
        else_: EffectsKey,
    },
    LoopFor {
        variable: SymbolKey,
        iterable: ValueKey,
        effects: EffectsKey,
    },
    LoopWhile {
        condition: ValueKey,
        effects: EffectsKey,
    },
    Scope {
        body: EffectsKey,
    },
    Call {
        value: ValueKey,
        args: ValueSliceKey,
    },
    MethodCall {
        table: ValueKey,
        name: StringKey,
        args: ValueSliceKey,
    },
    Return {
        value: ValueKey,
    },
    BreakLoop,
    ContinueLoop,
    NoEffectValue {
        value: ValueKey,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Branch {
        condition: ValueKey,
        then: EffectsKey,
        then_tail: ValueKey,
        else_: EffectsKey,
        else_tail: ValueKey,
    },
    Prefix {
        op: PrefixOp,
        value: ValueKey,
    },
    Binary {
        op: BinaryOp,
        lhs: ValueKey,
        rhs: ValueKey,
    },
    Call {
        value: ValueKey,
        args: ValueSliceKey,
    },
    Index {
        value: ValueKey,
        index: ValueKey,
    },
    Field {
        value: ValueKey,
        name: StringKey,
    },
    MethodCall {
        value: ValueKey,
        name: StringKey,
        args: ValueSliceKey,
    },
    Block {
        effects: EffectsKey,
        tail: ValueKey,
    },
    Local {
        name: SymbolKey,
    },
    Int(i64),
    Float(f64),
    String(UString),
    Bool(bool),
    Nil,
    Function(FunctionKey),
    Array {
        elements: ValueSliceKey,
    },
    Table {
        fields: Box<[(TableKeyName, ValueKey)]>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TableKeyName {
    Value(ValueKey),
    String(StringKey),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixOp {
    Plus(SyntaxToken),
    Minus(SyntaxToken),
    Not(SyntaxToken),
    BitNot(SyntaxToken),
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add(SyntaxToken),
    Sub(SyntaxToken),
    Mul(SyntaxToken),
    Div(SyntaxToken),
    Mod(SyntaxToken),
    Shl(SyntaxToken),
    Shr(SyntaxToken),
    Concat(SyntaxToken),
    Eq(SyntaxToken),
    Ne(SyntaxToken),
    Lt(SyntaxToken),
    Le(SyntaxToken),
    Gt(SyntaxToken),
    Ge(SyntaxToken),
    And(SyntaxToken),
    Or(SyntaxToken),
    BitAnd(SyntaxToken),
    BitOr(SyntaxToken),
    BitXor(SyntaxToken),
    Assign(SyntaxToken),
    Missing,
}
