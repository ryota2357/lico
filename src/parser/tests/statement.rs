mod common;

macro_rules! chunk_test {
    (name = $name:ident, source = $source:expr, expected = [$($lines:tt)*]) => {
        #[test]
        fn $name() {
            let program = common::parse_program($source);
            let res = program.body;
            let expected = vec![$($lines),*].join("\n");
            assert_eq!(res.to_string().trim(), expected);
        }
    };
}

chunk_test! {
    name = define_variable,
    source = "var x = 17",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    Var (s) @0..10"
        "      name: x @4..5"
        "      expr"
        "        Primitive (e) 17 @8..10"
    ]
}

chunk_test! {
    name = assign_variable,
    source = "x = true",
    expected = [
        "Chunk"
        "  captures: x @0..1"
        "  block"
        "    Assign (s) @0..8"
        "      name: x @0..1"
        "      accesser: None"
        "      expr"
        "        Primitive (e) true @4..8"
    ]
}

chunk_test! {
    name = assign_table_field,
    source = "x['y'].z = 10",
    expected = [
        "Chunk"
        "  captures: x @0..1"
        "  block"
        "    Assign (s) @0..13"
        "      name: x @0..1"
        "      accesser"
        "        Primitive (e) \"y\" @2..5"
        "        Primitive (e) \"z\" @7..8"
        "      expr"
        "        Primitive (e) 10 @11..13"
    ]
}

chunk_test! {
    name = define_function_without_args_and_body,
    source = "func f() end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    Func (s) @0..12"
        "      name: f @5..6"
        "      args: None"
        "      body"
        "        Chunk"
        "          captures: None"
        "          block: None"
    ]
}

chunk_test! {
    name = define_function_with_args_and_body,
    source = "func f(a, b) return 'a' end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    Func (s) @0..27"
        "      name: f @5..6"
        "      args"
        "        a @7..8"
        "        b @10..11"
        "      body"
        "        Chunk"
        "          captures: None"
        "          block"
        "            Return (s) @13..23"
        "              value"
        "                Primitive (e) \"a\" @20..23"
    ]
}

chunk_test! {
    name = define_function_with_trailing_comma,
    source = "func f(a,) end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    Func (s) @0..14"
        "      name: f @5..6"
        "      args"
        "        a @7..8"
        "      body"
        "        Chunk"
        "          captures: None"
        "          block: None"
    ]
}

chunk_test! {
    name = define_table_field_function,
    source = "func t.a.b() end",
    expected = [
        "Chunk"
        "  captures: t @5..6"
        "  block"
        "    FieldFunc (s) @0..16"
        "      table: t @5..6"
        "      fields"
        "        a @7..8"
        "        b @9..10"
        "      args: None"
        "      body"
        "        Chunk"
        "          captures: None"
        "          block: None"
    ]
}

chunk_test! {
    name = call_function_without_args,
    source = "f()",
    expected = [
        "Chunk"
        "  captures: f @0..1"
        "  block"
        "    Call (s) @0..3"
        "      expr"
        "        Local (e) \"f\" @0..1"
        "      accesser: None"
        "      args: None"
    ]
}

chunk_test! {
    name = call_table_function,
    source = "t.f(10, a)",
    expected = [
        "Chunk"
        "  captures: t @0..1"
        "  block"
        "    Call (s) @0..10"
        "      expr"
        "        Local (e) \"t\" @0..1"
        "      accesser"
        "        Primitive (e) \"f\" @2..3"
        "      args"
        "        Primitive (e) 10 @5..7"
        "        Local (e) \"a\" @9..10"
    ]
}
