use super::*;
use foundation::{object::UString, syntax::TextRange};

fn _size_check() {
    const {
        assert!(size_of::<ICodeSource>() == 32);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ICodeSource {
    LoadIntObject(i64),
    LoadFloatObject(f64),
    LoadStringObject(UString),
    LoadBoolObject(bool),
    LoadNilObject,
    LoadLocal(il::LocalId),

    Unload,

    StoreLocal(il::LocalId),
    StoreNewLocal,

    MakeArray(usize),

    // Exeption
    // - The type of the key is not a string.
    // ---
    // .1: The text range of table keys.
    //     If the key is ensured to be type of string, the text range is None.
    MakeTable(usize, Box<[Option<TextRange>]>),

    DropLocal(usize),

    Jump(isize),
    JumpIfTrue(isize),
    JumpIfFalse(isize),

    // Exeption
    // - The callee is not type of Function or RustFunction or Table.
    // - No `__call` method defined for the popped table type value.
    // - The number of calee arguments is not equal to the specified argument count.
    // ---
    // .1: Callee text range
    // .2: Each argument text range
    Call(u8, TextRange, Box<[TextRange]>),

    // Exeption
    // - The receiver is not type of Table.
    // - The specified method name is not defined in the receiver table object.
    // - The number of method arguments is not equal to the specified argument count.
    // ---
    // .2: first is Method name text range, rest is each argument text range
    CallMethod(u8, UString, Box<[TextRange]>),

    // Exeption
    // - The container is not type of Table or Array.
    // - The key is not type of Int if the container is Array.
    // - The key is not type of String if the container is Table.
    // ---
    // .0: The key text range
    SetItem(TextRange),

    // Exeption
    // - The container is not type of Table or Array.
    // - The key is not type of Int if the container is Array.
    // - The key is not type of String if the container is Table.
    // ---
    // .0: The key text range
    GetItem(TextRange),

    // Exeption
    // The container is not type of Table.
    // --
    // .1: The container text range
    SetMethod(UString, TextRange),

    // Exeption
    // - Popped values are not type of Int or Float or Table.
    // - No `__***` method defined for the popped table type value.
    // - Division by zero.
    // ---
    // .0: The operator text range
    Add(TextRange),
    Sub(TextRange),
    Mul(TextRange),
    Div(TextRange),
    Mod(TextRange),
    Unm(TextRange),
    Unp(TextRange),
    Not(TextRange),
    Eq(TextRange),
    NotEq(TextRange),
    Less(TextRange),
    LessEq(TextRange),
    Greater(TextRange),
    GreaterEq(TextRange),
    Concat(TextRange),
    BitAnd(TextRange),
    BitOr(TextRange),
    BitXor(TextRange),
    BitNot(TextRange),
    ShiftL(TextRange),
    ShiftR(TextRange),

    GetIter,
    IterMoveNext,
    IterCurrent,

    BeginFuncSection,
    FuncSetProperty(u8, FunctionListId),
    FuncAddCapture(il::LocalId),
    EndFuncSection,

    // Nop,
    Leave,

    Tombstone,
}
