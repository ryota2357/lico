#[derive(Clone, Debug, PartialEq)]
pub enum Code<'src> {
    LoadInt(i64),
    LoadFloat(f64),
    LoadBool(bool),
    LoadString(String),
    LoadStringAsRef(&'src str),
    LoadNil,
    LoadLocal(&'src str),
    LoadRustFunction(fn(&[crate::Object<'src>]) -> Result<crate::Object<'src>, String>),
    UnloadTop,

    SetLocal(&'src str),
    MakeLocal(&'src str),
    MakeArray(u32),
    MakeNamed(&'src str),
    MakeExprNamed,
    MakeTable(u32),
    DropLocal(usize),

    Jump(isize),
    JumpIfTrue(isize),
    JumpIfFalse(isize),

    CustomMethod(&'src str, u8),
    Call(u8),
    SetItem,
    GetItem,
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    Mod,       // %
    Pow,       // *
    Unm,       // - (unary)
    Eq,        // ==
    NotEq,     // !=
    Less,      // <
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=

    Builtin(BuiltinInstr, u8),

    BeginFuncCreation,
    AddCapture(&'src str),
    AddArgument(&'src str),
    EndFuncCreation,

    Nop,
    Return,

    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinInstr {
    Write,
    Flush,
}
