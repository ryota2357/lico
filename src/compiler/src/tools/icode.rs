use vm::code::BuiltinInstr;

use super::*;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum ICode {
    LoadInt(i64),
    LoadFloat(f64),
    LoadBool(bool),
    LoadString(String),
    LoadNil,
    LoadLocal(VariableId),
    UnloadTop,

    SetLocal(VariableId),
    MakeLocal,
    MakeArray(u32),
    MakeNamed,
    MakeTable(u32),
    DropLocal(usize),

    Jump(isize),
    JumpIfTrue(isize),
    JumpIfFalse(isize),

    CallMethod(Cow<'static, str>, u8, Span),
    Call(u8, Span),
    SetItem(Span),
    GetItem(Span),
    Add(Span),       // +
    Sub(Span),       // -
    Mul(Span),       // *
    Div(Span),       // /
    Mod(Span),       // %
    Pow(Span),       // *
    Unm(Span),       // - (unary)
    Eq(Span),        // ==
    NotEq(Span),     // !=
    Less(Span),      // <
    LessEq(Span),    // <=
    Greater(Span),   // >
    GreaterEq(Span), // >=
    Concat(Span),    // ..

    Builtin(BuiltinInstr, u8),

    BeginFuncCreation,
    AddCapture(VariableId),
    AddArgument(()),
    EndFuncCreation,

    Placeholder,

    Nop,
    Return,
}
