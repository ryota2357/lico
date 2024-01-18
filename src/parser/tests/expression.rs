mod common;
use pretty_assertions::assert_eq;

macro_rules! expression_test {
    (name = $name:ident, source = $source:expr, expected = [$($lines:tt)*]) => {
        #[test]
        fn $name() {
            use foundation::ast::Statement;
            fn decrement_ranges(input: &str, amount: u32) -> String {
                let re = regex::Regex::new(r"@(\d+)\.\.(\d+)").unwrap();
                let result = re.replace_all(input, |captures: &regex::Captures| {
                    let start: u32 = captures[1].parse().unwrap();
                    let end: u32 = captures[2].parse().unwrap();
                    format!("@{}..{}", start - amount, end - amount)
                });
                result.to_string()
            }
            let src = format!("_={}", $source);
            let program = common::parse_program(&src);
            let stats = program.body.block;
            assert_eq!(stats.len(), 1);
            let statement = &stats[0];
            let expected = vec![$($lines),*].join("\n");
            if let Statement::Assign { expr, .. } = &statement.0 {
                assert_eq!(decrement_ranges(expr.0.to_string().trim(), 2), expected);
            } else {
                panic!(
                    "Expected Statement::Assign, but got {:?}",
                    statement
                );
            }
        }
    };
}

expression_test! {
    name = empty_function_object,
    source = "func() end",
    expected = [
        "FunctionObject (e)"
        "  args: None"
        "  body"
        "    Chunk"
        "      captures: None"
        "      block: None"

    ]
}

expression_test! {
    name = function_object_with_args_trailing_comma,
    source = "func(a,) end",
    expected = [
        "FunctionObject (e)"
        "  args"
        "    a @5..6"
        "  body"
        "    Chunk"
        "      captures: None"
        "      block: None"
    ]
}

expression_test! {
    name = function_object_with_args_and_body,
    source = "func(a, in b) return c end",
    expected = [
        "FunctionObject (e)"
        "  args"
        "    a @5..6"
        "    b [in] @11..12"
        "  body"
        "    Chunk"
        "      captures: c @21..22"
        "      block"
        "        Return (s) @14..22"
        "          value"
        "            Local (e) c @21..22"
    ]
}

expression_test! {
    name = empty_array_object,
    source = "[]",
    expected = [ "ArrayObject (e)" ]
}

expression_test! {
    name = array_object_with_trailing_comma,
    source = "[1,]",
    expected = [
        "ArrayObject (e)"
        "  000"
        "    Primitive (e) 1 @1..2"
    ]
}

expression_test! {
    name = nested_array_object,
    source = "[12, [true, 'a']]",
    expected = [
        "ArrayObject (e)"
        "  000"
        "    Primitive (e) 12 @1..3"
        "  001"
        "    ArrayObject (e) @5..16"
        "      000"
        "        Primitive (e) true @6..10"
        "      001"
        "        Primitive (e) \"a\" @12..15"
    ]
}

expression_test! {
    name = empty_table_object,
    source = "{}",
    expected = [ "TableObject (e)" ]
}

expression_test! {
    name = table_object_with_trailing_comma,
    source = "{a = 1,}",
    expected = [
        "TableObject (e)"
        "  key: a @1..2"
        "  value"
        "    Primitive (e) 1 @5..6"
    ]
}

expression_test! {
    name = nested_table_object,
    source = "{a = 1, b = {c = 2}}",
    expected = [
        "TableObject (e)"
        "  key: a @1..2"
        "  value"
        "    Primitive (e) 1 @5..6"
        "  key: b @8..9"
        "  value"
        "    TableObject (e) @12..19"
        "      key: c @13..14"
        "      value"
        "        Primitive (e) 2 @17..18"
    ]
}

expression_test! {
    name = complicated_func_with_trailing_comma,
    source = "f(g(),)",
    expected = [
        "Call (e)"
        "  expr"
        "    Local (e) f @0..1"
        "  args"
        "    Call (e) @2..5"
        "      expr"
        "        Local (e) g @2..3"
        "      args: None"
    ]
}

// TODO: error test
// expression_test! {
//     name = call_with_only_comma,
//     source = "f(,)",
//     expected = [
//         "Call (e)"
//         "  expr"
//         "    Local (e) f @0..1"
//         "  args"
//     ]
// }

expression_test! {
    name = multiple_call,
    source = "f(1)(2)(3)",
    expected = [
        "Call (e)"
        "  expr"
        "    Call (e) @0..7"
        "      expr"
        "        Call (e) @0..4"
        "          expr"
        "            Local (e) f @0..1"
        "          args"
        "            Primitive (e) 1 @2..3"
        "      args"
        "        Primitive (e) 2 @5..6"
        "  args"
        "    Primitive (e) 3 @8..9"
    ]
}

expression_test! {
    name = method_chain,
    source = "a->b()->c()",
    expected = [
        "MethodCall (e)"
        "  expr"
        "    MethodCall (e) @0..6"
        "      expr"
        "        Local (e) a @0..1"
        "      name: b @3..4"
        "      args: None"
        "  name: c @8..9"
        "  args: None"
    ]
}

expression_test! {
    name = anonymous_func_call,
    source = "(func(x) return x end)(1)",
    expected = [
        "Call (e)"
        "  expr"
        "    FunctionObject (e) @0..22"
        "      args"
        "        x @6..7"
        "      body"
        "        Chunk"
        "          captures: None"
        "          block"
        "            Return (s) @9..17"
        "              value"
        "                Local (e) x @16..17"
        "  args"
        "    Primitive (e) 1 @23..24"
    ]
}

expression_test! {
    name = delimited_call,
    source = "(f())",
    expected = [
        "Call (e)"
        "  expr"
        "    Local (e) f @1..2"
        "  args: None"
    ]
}

expression_test! {
    name = more_delimited_call,
    source = "(((f()))())",
    expected = [
        "Call (e)"
        "  expr"
        "    Call (e) @1..8"
        "      expr"
        "        Local (e) f @3..4"
        "      args: None"
        "  args: None"
    ]
}

expression_test! {
    name = delimited_primitive,
    source = "(1)",
    expected = [
        "Primitive (e) 1 @1..2"
    ]
}

expression_test! {
    name = delimited_local,
    source = "(a)",
    expected = [
        "Local (e) a @1..2"
    ]
}

expression_test! {
    name = four_arithmetic,
    source = "-1 + 2 * 3 / 4 - 5", // ((-1) + ((2 * 3) / 4)) - 5
    expected = [
        "Binary (e)"
        "  op: -"
        "  lhs"
        "    Binary (e) @0..14"
        "      op: +"
        "      lhs"
        "        Primitive (e) -1 @0..2"
        "      rhs"
        "        Binary (e) @5..14"
        "          op: /"
        "          lhs"
        "            Binary (e) @5..10"
        "              op: *"
        "              lhs"
        "                Primitive (e) 2 @5..6"
        "              rhs"
        "                Primitive (e) 3 @9..10"
        "          rhs"
        "            Primitive (e) 4 @13..14"
        "  rhs"
        "    Primitive (e) 5 @17..18"
    ]
}

expression_test! {
    name = four_arithmetic_with_parentheses,
    source = "-((1 + 2) * 3) / 4",
    expected = [
        "Binary (e)"
        "  op: /"
        "  lhs"
        "    Unary (e) @0..14"
        "      op: -"
        "      expr"
        "        Binary (e) @1..14"
        "          op: *"
        "          lhs"
        "            Binary (e) @2..9"
        "              op: +"
        "              lhs"
        "                Primitive (e) 1 @3..4"
        "              rhs"
        "                Primitive (e) 2 @7..8"
        "          rhs"
        "            Primitive (e) 3 @12..13"
        "  rhs"
        "    Primitive (e) 4 @17..18"
    ]
}

expression_test! {
    name = logical_op,
    source = "not a == 10 or 5 >= b and false", // ((not a) == 10) or ((5 >= b) and false)
    expected = [
        "Binary (e)"
        "  op: or"
        "  lhs"
        "    Binary (e) @0..11"
        "      op: =="
        "      lhs"
        "        Unary (e) @0..5"
        "          op: not"
        "          expr"
        "            Local (e) a @4..5"
        "      rhs"
        "        Primitive (e) 10 @9..11"
        "  rhs"
        "    Binary (e) @15..31"
        "      op: and"
        "      lhs"
        "        Binary (e) @15..21"
        "          op: >="
        "          lhs"
        "            Primitive (e) 5 @15..16"
        "          rhs"
        "            Local (e) b @20..21"
        "      rhs"
        "        Primitive (e) false @26..31"
    ]
}

expression_test! {
    name = logical_op_with_parentheses,
    source = "(not (a != 10) or 5 < b) and false",
    expected = [
        "Binary (e)"
        "  op: and"
        "  lhs"
        "    Binary (e) @0..24"
        "      op: or"
        "      lhs"
        "        Unary (e) @1..14"
        "          op: not"
        "          expr"
        "            Binary (e) @5..14"
        "              op: !="
        "              lhs"
        "                Local (e) a @6..7"
        "              rhs"
        "                Primitive (e) 10 @11..13"
        "      rhs"
        "        Binary (e) @18..23"
        "          op: <"
        "          lhs"
        "            Primitive (e) 5 @18..19"
        "          rhs"
        "            Local (e) b @22..23"
        "  rhs"
        "    Primitive (e) false @29..34"
    ]
}

expression_test! {
    name = string_concat,
    source = "'a' .. 4 + 1 == 'a5'",
    expected = [
        "Binary (e)"
        "  op: =="
        "  lhs"
        "    Binary (e) @0..12"
        "      op: .."
        "      lhs"
        "        Primitive (e) \"a\" @0..3"
        "      rhs"
        "        Binary (e) @7..12"
        "          op: +"
        "          lhs"
        "            Primitive (e) 4 @7..8"
        "          rhs"
        "            Primitive (e) 1 @11..12"
        "  rhs"
        "    Primitive (e) \"a5\" @16..20"
    ]
}

expression_test! {
    name = complicated_pratt,
    source = "-(false or b).c[c.c() and -d()] * 2",
    expected = [
        "Binary (e)"
        "  op: *"
        "  lhs"
        "    Unary (e) @0..31"
        "      op: -"
        "      expr"
        "        IndexAccess (e) @1..31"
        "          expr"
        "            DotAccess (e) @1..15"
        "              expr"
        "                Binary (e) @1..13"
        "                  op: or"
        "                  lhs"
        "                    Primitive (e) false @2..7"
        "                  rhs"
        "                    Local (e) b @11..12"
        "              accessor: c @14..15"
        "          accessor"
        "            Binary (e) @16..30"
        "              op: and"
        "              lhs"
        "                Call (e) @16..21"
        "                  expr"
        "                    DotAccess (e) @16..19"
        "                      expr"
        "                        Local (e) c @16..17"
        "                      accessor: c @18..19"
        "                  args: None"
        "              rhs"
        "                Unary (e) @26..30"
        "                  op: -"
        "                  expr"
        "                    Call (e) @27..30"
        "                      expr"
        "                        Local (e) d @27..28"
        "                      args: None"
        "  rhs"
        "    Primitive (e) 2 @34..35"
    ]
}
