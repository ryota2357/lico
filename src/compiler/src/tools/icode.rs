use lexer::TextSpan;
use vm::code::{ArgumentKind, BuiltinInstr};

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

    CallMethod(Cow<'static, str>, u8, TextSpan),
    Call(u8, TextSpan),
    SetItem(TextSpan),
    GetItem(TextSpan),
    Add(TextSpan),       // +
    Sub(TextSpan),       // -
    Mul(TextSpan),       // *
    Div(TextSpan),       // /
    Mod(TextSpan),       // %
    Unm(TextSpan),       // - (unary)
    Eq(TextSpan),        // ==
    NotEq(TextSpan),     // !=
    Less(TextSpan),      // <
    LessEq(TextSpan),    // <=
    Greater(TextSpan),   // >
    GreaterEq(TextSpan), // >=
    Concat(TextSpan),    // ..

    Builtin(BuiltinInstr, u8),

    BeginFuncCreation,
    AddCapture(VariableId),
    AddArgument(ArgumentKind),
    EndFuncCreation,

    Placeholder,

    Nop,
    Return,
}
