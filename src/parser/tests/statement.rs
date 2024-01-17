mod common;
use pretty_assertions::assert_eq;

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
        "    FieldAssign (s) @0..13"
        "      table"
        "        IndexAccess (e) @0..6"
        "          expr"
        "            Local (e) x @0..1"
        "          accessor"
        "            Primitive (e) \"y\" @2..5"
        "      field"
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
        "        Local (e) f @0..1"
        "      args: None"
    ]
}

chunk_test! {
    name = call_table_function,
    source = "t.f(10, a)",
    expected = [
        "Chunk"
        "  captures"
        "    a @8..9"
        "    t @0..1"
        "  block"
        "    Call (s) @0..10"
        "      expr"
        "        DotAccess (e) @0..3"
        "          expr"
        "            Local (e) t @0..1"
        "          accessor: f @2..3"
        "      args"
        "        Primitive (e) 10 @4..6"
        "        Local (e) a @8..9"
    ]
}

chunk_test! {
    name = method_call,
    source = "a->b('a')",
    expected = [
        "Chunk"
        "  captures: a @0..1"
        "  block"
        "    MethodCall (s) @0..9"
        "      expr"
        "        Local (e) a @0..1"
        "      name: b @3..4"
        "      args"
        "        Primitive (e) \"a\" @5..8"
    ]
}

chunk_test! {
    name = method_call_obj,
    source = "[1, 2]->len()",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    MethodCall (s) @0..13"
        "      expr"
        "        ArrayObject (e) @0..6"
        "          000"
        "            Primitive (e) 1 @1..2"
        "          001"
        "            Primitive (e) 2 @4..5"
        "      name: len @8..11"
        "      args: None"
    ]
}

chunk_test! {
    name = if_without_body,
    source = "if true then end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    If (s) @0..16"
        "      cond"
        "        Primitive (e) true @3..7"
        "      body"
        "        Block"
    ]
}

chunk_test! {
    name = if_with_body,
    source = "if true then return end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    If (s) @0..23"
        "      cond"
        "        Primitive (e) true @3..7"
        "      body"
        "        Block"
        "          Return (s) @13..19"
    ]
}

chunk_test! {
    name = if_else,
    source = "if true then return else end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    If (s) @0..28"
        "      cond"
        "        Primitive (e) true @3..7"
        "      body"
        "        Block"
        "          Return (s) @13..19"
        "      else"
        "        Block"
    ]
}

chunk_test! {
    name = if_elif,
    source = "if true then elif false then f() end",
    expected = [
        "Chunk"
        "  captures: f @29..30"
        "  block"
        "    If (s) @0..36"
        "      cond"
        "        Primitive (e) true @3..7"
        "      body"
        "        Block"
        "      elif"
        "        cond"
        "          Primitive (e) false @18..23"
        "        body"
        "          Block"
        "            Call (s) @29..32"
        "              expr"
        "                Local (e) f @29..30"
        "              args: None"
    ]
}

chunk_test! {
    name = if_elif_elif,
    source = "if true then elif false then return 1 elif true then return 2 end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    If (s) @0..65"
        "      cond"
        "        Primitive (e) true @3..7"
        "      body"
        "        Block"
        "      elif"
        "        cond"
        "          Primitive (e) false @18..23"
        "        body"
        "          Block"
        "            Return (s) @29..37"
        "              value"
        "                Primitive (e) 1 @36..37"
        "      elif"
        "        cond"
        "          Primitive (e) true @43..47"
        "        body"
        "          Block"
        "            Return (s) @53..61"
        "              value"
        "                Primitive (e) 2 @60..61"
    ]
}

chunk_test! {
    name = if_elif_else,
    source = "if true then return 1 elif false then return 2 else return 3 end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    If (s) @0..64"
        "      cond"
        "        Primitive (e) true @3..7"
        "      body"
        "        Block"
        "          Return (s) @13..21"
        "            value"
        "              Primitive (e) 1 @20..21"
        "      elif"
        "        cond"
        "          Primitive (e) false @27..32"
        "        body"
        "          Block"
        "            Return (s) @38..46"
        "              value"
        "                Primitive (e) 2 @45..46"
        "      else"
        "        Block"
        "          Return (s) @52..60"
        "            value"
        "              Primitive (e) 3 @59..60"
    ]
}

chunk_test! {
    name = for_in_array_without_body,
    source = "for i in [1, 2, 3] do end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    For (s) @0..25"
        "      value: i @4..5"
        "      iter"
        "        ArrayObject (e) @9..18"
        "          000"
        "            Primitive (e) 1 @10..11"
        "          001"
        "            Primitive (e) 2 @13..14"
        "          002"
        "            Primitive (e) 3 @16..17"
        "      body"
        "        Block"
    ]
}

chunk_test! {
    name = for_in_expr_with_body,
    source = "for i in 1->upto(10) do a = a + i end",
    expected = [
        "Chunk"
        "  captures: a @28..29"
        "  block"
        "    For (s) @0..37"
        "      value: i @4..5"
        "      iter"
        "        MethodCall (e) @9..20"
        "          expr"
        "            Primitive (e) 1 @9..10"
        "          name: upto @12..16"
        "          args"
        "            Primitive (e) 10 @17..19"
        "      body"
        "        Block"
        "          Assign (s) @24..33"
        "            name: a @24..25"
        "            expr"
        "              Binary (e) @28..33"
        "                op: +"
        "                lhs"
        "                  Local (e) a @28..29"
        "                rhs"
        "                  Local (e) i @32..33"
    ]
}

chunk_test! {
    name = while_loop,
    source = "while ok() do break end",
    expected = [
        "Chunk"
        "  captures: ok @6..8"
        "  block"
        "    While (s) @0..23"
        "      cond"
        "        Call (e) @6..10"
        "          expr"
        "            Local (e) ok @6..8"
        "          args: None"
        "      body"
        "        Block"
        "          Break (s) @14..19"
    ]
}

chunk_test! {
    name = do_without_body,
    source = "do end",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    Do (s) @0..6"
        "      body"
        "        Block"
    ]
}

chunk_test! {
    name = return_none,
    source = "return",
    expected = [
        "Chunk"
        "  captures: None"
        "  block"
        "    Return (s) @0..6"
    ]
}
