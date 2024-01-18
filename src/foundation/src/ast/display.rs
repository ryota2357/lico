use super::*;

macro_rules! impl_display {
    ($($ty:ty),*) => {
        $(
            impl ::std::fmt::Display for $ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    let mut builder = PrettyPrintBuilder::new(0);
                    builder.nest(0, self);
                    write!(f, "{}", builder.into_string())
                }
            }
        )*
    };
}
impl_display!(Program<'_>, Chunk<'_>, Block<'_>, Expression<'_>);

trait PrettyPrint {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder);
}

struct PrettyPrintBuilder {
    indent: usize,
    result: String,
}

impl PrettyPrintBuilder {
    fn new(indent: usize) -> Self {
        Self {
            indent,
            result: String::new(),
        }
    }

    fn append(&mut self, indent: usize, content: impl AsRef<str>) {
        let indent = " ".repeat(self.indent + indent);
        self.result.push_str(&(indent + content.as_ref()));
        self.result.push('\n');
    }

    fn append_inline(&mut self, content: impl AsRef<str>) {
        if let Some('\n') = self.result.chars().last() {
            self.result.pop();
        }
        self.result.push_str(content.as_ref());
        self.result.push('\n');
    }

    fn nest(&mut self, indent: usize, target: &impl PrettyPrint) {
        self.indent += indent;
        target.pretty_print(self);
        self.indent -= indent;
    }

    fn into_string(self) -> String {
        self.result
    }
}

// Program
//   attributes
//      name @[1..2, 5..10]
//   body
impl PrettyPrint for Program<'_> {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        builder.append(0, "Program");
        match self.attributes.len() {
            0 => {
                builder.append(2, "attributes: None");
            }
            1 => {
                let (name, spans) = &self.attributes[0];
                builder.append(
                    2,
                    format!(
                        "attributes: {} @[{}]",
                        name,
                        spans
                            .iter()
                            .map(|span| span.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            }
            2.. => {
                builder.append(2, "attributes");
                for attribute in &self.attributes {
                    let (name, spans) = attribute;
                    builder.append(
                        4,
                        format!(
                            "{} @[{}]",
                            name,
                            spans
                                .iter()
                                .map(|span| span.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    )
                }
            }
        }
        builder.append(2, "body");
        builder.nest(4, &self.body);
    }
}

// Chunk
//   captures
//     name @1
//   statements
//     Var @1..2
fn chunk_pretty_print_inner(builder: &mut PrettyPrintBuilder, chunk: &Chunk<'_>) {
    match chunk.captures.len() {
        0 => {
            builder.append(2, "captures: None");
        }
        1 => {
            let capture = &chunk.captures[0];
            builder.append(2, format!("captures: {} @{}", capture.0, capture.1));
        }
        2.. => {
            builder.append(2, "captures");
            for capture in &chunk.captures {
                builder.append(4, format!("{} @{}", capture.0, capture.1))
            }
        }
    }
    if chunk.block.is_empty() {
        builder.append(2, "block: None");
    } else {
        builder.append(2, "block");
        for stmt in chunk.block.iter() {
            builder.nest(4, stmt);
        }
    }
}
impl PrettyPrint for Chunk<'_> {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        builder.append(0, "Chunk");
        chunk_pretty_print_inner(builder, self);
    }
}
impl PrettyPrint for (Chunk<'_>, TextSpan) {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        let (chunk, span) = self;
        builder.append(0, format!("Chunk @{}", span));
        chunk_pretty_print_inner(builder, chunk);
    }
}

// Block
//   Var @1..2
fn block_pretty_print_inner(builder: &mut PrettyPrintBuilder, block: &Block<'_>) {
    if block.is_empty() {
        return;
    }
    for stmt in block.iter() {
        builder.nest(2, stmt);
    }
}
impl PrettyPrint for Block<'_> {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        builder.append(0, "Block");
        block_pretty_print_inner(builder, self);
    }
}
impl PrettyPrint for (Block<'_>, TextSpan) {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        let (block, span) = self;
        builder.append(0, format!("Block @{}", span));
        block_pretty_print_inner(builder, block);
    }
}

impl PrettyPrint for (Statement<'_>, TextSpan) {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        let (stmt, span) = self;
        match stmt {
            // Var (s) @1..2
            //   name: [name] @1..2
            //   expr
            //     [expr]
            Statement::Var { name, expr } => {
                builder.append(0, format!("Var (s) @{}", span));
                builder.append(2, format!("name: {} @{}", name.0, name.1));
                builder.append(2, "expr");
                builder.nest(4, expr);
            }

            // Func (s) @1..2
            //   name: [name] @1..2
            //   args
            //     [annotation] name @1..2
            //   body
            Statement::Func { name, args, body } => {
                builder.append(0, format!("Func (s) @{}", span));
                builder.append(2, format!("name: {} @{}", name.0, name.1));
                if args.is_empty() {
                    builder.append(2, "args: None");
                } else {
                    builder.append(2, "args");
                    for (annotation, name, span) in args {
                        let annotation = match annotation {
                            FunctArgAnnotation::None => "",
                            FunctArgAnnotation::Ref => "[ref] ",
                            FunctArgAnnotation::In => "[in] ",
                        };
                        builder.append(4, &format!("{}{} @{}", annotation, name, span));
                    }
                }
                builder.append(2, "body");
                builder.nest(4, body);
            }

            // FieldFunc (s) @1..2
            //   table: [name] @1..2
            //   fields
            //     name @1..2
            //   args
            //     [annotation] name @1
            //   body
            Statement::FieldFunc {
                table,
                fields,
                args,
                body,
            } => {
                builder.append(0, format!("FieldFunc (s) @{}", span));
                builder.append(2, format!("table: {} @{}", table.0, table.1));
                if fields.is_empty() {
                    builder.append(2, "fields: None");
                } else {
                    builder.append(2, "fields");
                    for (name, span) in fields {
                        builder.append(4, format!("{} @{}", name, span));
                    }
                }
                if args.is_empty() {
                    builder.append(2, "args: None");
                } else {
                    builder.append(2, "args");
                    for (annotation, name, span) in args {
                        let annotation = match annotation {
                            FunctArgAnnotation::None => "",
                            FunctArgAnnotation::Ref => "[ref]",
                            FunctArgAnnotation::In => "[in]",
                        };
                        builder.append(4, format!("{} {} @{}", annotation, name, span));
                    }
                }
                builder.append(2, "body");
                builder.nest(4, body);
            }

            // Assign (s) @1..2
            //   name: [name] @1
            //   expr
            //     [expr]
            Statement::Assign { name, expr } => {
                builder.append(0, format!("Assign (s) @{}", span));
                builder.append(2, format!("name: {} @{}", name.0, name.1));
                builder.append(2, "expr");
                builder.nest(4, expr);
            }

            // FieldAssign (s) @1..2
            //   table
            //     [expr]
            //   field
            //     [expr] @1
            //   expr
            //     [expr]
            Statement::FieldAssign { table, field, expr } => {
                builder.append(0, format!("FieldAssign (s) @{}", span));
                builder.append(2, "table");
                builder.nest(4, table);
                builder.append(2, "field");
                builder.nest(4, field);
                builder.append(2, "expr");
                builder.nest(4, expr);
            }

            // If (s) @1..2
            //   cond
            //     [expr]
            //   body
            //     [block]
            //   elif
            //      cond
            //        [expr]
            //      body
            //        [block]
            //   else
            //     [block]
            Statement::If {
                cond,
                body,
                elifs,
                else_,
            } => {
                builder.append(0, format!("If (s) @{}", span));
                builder.append(2, "cond");
                builder.nest(4, cond);
                builder.append(2, "body");
                builder.nest(4, body);
                if !elifs.is_empty() {
                    for (cond, body) in elifs {
                        builder.append(2, "elif");
                        builder.append(4, "cond");
                        builder.nest(6, cond);
                        builder.append(4, "body");
                        builder.nest(6, body);
                    }
                }
                if let Some(body) = else_ {
                    builder.append(2, "else");
                    builder.nest(4, body);
                }
            }

            // For (s) @1..2
            //   value: [name] @1..2
            //   iter
            //     [expr]
            //   body
            //     [block]
            Statement::For { value, iter, body } => {
                builder.append(0, format!("For (s) @{}", span));
                builder.append(2, format!("value: {} @{}", value.0, value.1));
                builder.append(2, "iter");
                builder.nest(4, iter);
                builder.append(2, "body");
                builder.nest(4, body);
            }

            // While (s) @1..2
            //   cond
            //     [expr]
            //   body
            //     [block]
            Statement::While { cond, body } => {
                builder.append(0, format!("While (s) @{}", span));
                builder.append(2, "cond");
                builder.nest(4, cond);
                builder.append(2, "body");
                builder.nest(4, body);
            }

            // Do (s) @1..2
            //   body
            //     [block]
            Statement::Do { body } => {
                builder.append(0, format!("Do (s) @{}", span));
                builder.append(2, "body");
                builder.nest(4, body);
            }

            // Return (s) @1..2
            //   value
            //     [expr]
            Statement::Return { value } => {
                builder.append(0, format!("Return (s) @{}", span));
                if let Some(value) = value {
                    builder.append(2, "value");
                    builder.nest(4, value);
                }
            }

            // Continue (s) @1..2
            Statement::Continue => {
                builder.append(0, format!("Continue (s) @{}", span));
            }

            // Break (s) @1..2
            Statement::Break => {
                builder.append(0, format!("Break (s) @{}", span));
            }

            // Call (s) @1..2
            //   expr
            //     [expr]
            //   args
            //     [expr]
            Statement::Call { expr, args } => {
                builder.append(0, format!("Call (s) @{}", span));
                builder.append(2, "expr");
                builder.nest(4, expr);
                if args.is_empty() {
                    builder.append(2, "args: None");
                } else {
                    builder.append(2, "args");
                    for arg in args {
                        builder.nest(4, arg);
                    }
                }
            }

            // MethodCall (s) @1..2
            //   expr
            //     [expr]
            //   name: [name] @1
            //   args
            //     [expr]
            Statement::MethodCall { expr, name, args } => {
                builder.append(0, format!("MethodCall (s) @{}", span));
                builder.append(2, "expr");
                builder.nest(4, expr);
                builder.append(2, format!("name: {} @{}", name.0, name.1));
                if args.is_empty() {
                    builder.append(2, "args: None");
                } else {
                    builder.append(2, "args");
                    for arg in args {
                        builder.nest(4, arg);
                    }
                }
            }

            // Attribute (s) @1..2
            //   name: [name] @1..2
            //   args
            //     [name] @1..2
            Statement::Attribute { name, args } => {
                builder.append(0, format!("Attribute (s) @{}", span));
                builder.append(2, format!("name: {} @{}", name.0, name.1));
                if let Some(args) = args {
                    if args.is_empty() {
                        builder.append(2, "args: None");
                    } else {
                        builder.append(2, "args");
                        for arg in args {
                            builder.append(4, format!("{} @{}", arg.0, arg.1));
                        }
                    }
                }
            }

            // Error (s) @1..2
            Statement::Error => {
                builder.append(0, format!("Error (s) @{}", span));
            }
        }
    }
}

fn expression_name(expr: &Expression<'_>) -> &'static str {
    match expr {
        Expression::Unary { .. } => "Unary (e)",
        Expression::Binary { .. } => "Binary (e)",
        Expression::Local(_, _) => "Local (e)",
        Expression::Primitive(_, _) => "Primitive (e)",
        Expression::TableObject(_) => "TableObject (e)",
        Expression::ArrayObject(_) => "ArrayObject (e)",
        Expression::FunctionObject(_) => "FunctionObject (e)",
        Expression::Call { .. } => "Call (e)",
        Expression::MethodCall { .. } => "MethodCall (e)",
        Expression::IndexAccess { .. } => "IndexAccess (e)",
        Expression::DotAccess { .. } => "DotAccess (e)",
        Expression::Error => "Error (e)",
    }
}
fn expression_pretty_print_inner(builder: &mut PrettyPrintBuilder, expr: &Expression<'_>) {
    match expr {
        // Unary (e) @1..2
        //  op: [op]
        //  expr
        //    [expr]
        Expression::Unary { op, expr } => {
            let op = match op {
                UnaryOp::Not => "not",
                UnaryOp::Neg => "-",
            };
            builder.append(2, format!("op: {}", op));
            builder.append(2, "expr");
            builder.nest(4, expr);
        }

        // Binary (e) @1..2
        //   op: [op]
        //   lhs
        //     [expr]
        //   rhs
        //     [expr]
        Expression::Binary { op, lhs, rhs } => {
            let op = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Mod => "%",
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
                BinaryOp::Eq => "==",
                BinaryOp::NotEq => "!=",
                BinaryOp::Less => "<",
                BinaryOp::LessEq => "<=",
                BinaryOp::Greater => ">",
                BinaryOp::GreaterEq => ">=",
                BinaryOp::Concat => "..",
            };
            builder.append(2, format!("op: {}", op));
            builder.append(2, "lhs");
            builder.nest(4, lhs);
            builder.append(2, "rhs");
            builder.nest(4, rhs);
        }

        // Local (e) [name] @1..2
        Expression::Local(name, span) => {
            builder.append_inline(format!(" {} @{}", name, span));
        }

        // Primitive (e) @1..2
        Expression::Primitive(primitive, span) => match primitive {
            Primitive::Int(x) => builder.append_inline(format!(" {} @{}", x, span)),
            Primitive::Float(x) => builder.append_inline(format!(" {:.8} @{}", x, span)),
            Primitive::String(x) => builder.append_inline(format!(r#" "{}" @{}"#, x, span)),
            Primitive::Bool(x) => builder.append_inline(format!(" {} @{}", x, span)),
            Primitive::Nil => builder.append_inline(format!(" nil @{}", span)),
        },

        // TableObject (e) @1..2
        //   key
        //     [expr]
        //   value
        //     [expr]
        Expression::TableObject(table) => {
            for (key, value) in table.iter() {
                match key {
                    TableFieldKey::Ident(name, span) => {
                        builder.append(2, format!("key: {} @{}", name, span));
                    }
                    TableFieldKey::Expr(expr, span) => {
                        builder.append(2, "key");
                        builder.nest(4, &(expr, *span));
                    }
                }
                builder.append(2, "value");
                builder.nest(4, value);
            }
        }

        // ArrayObject (e) @1..2
        //   000
        //     [expr]
        Expression::ArrayObject(array) => {
            let idx_len = array.len().to_string().len().max(3);
            for (i, expr) in array.iter().enumerate() {
                builder.append(2, format!("{:0>width$}", i, width = idx_len));
                builder.nest(4, expr);
            }
        }

        // FunctionObject (e) @1..2
        //   args
        //     [annotation] name @1
        //   body
        //     [block]
        Expression::FunctionObject(function) => {
            if function.args.is_empty() {
                builder.append(2, "args: None");
            } else {
                builder.append(2, "args");
                for (annotation, name, span) in &function.args {
                    let annotation = match annotation {
                        FunctArgAnnotation::None => "",
                        FunctArgAnnotation::Ref => " [ref]",
                        FunctArgAnnotation::In => " [in]",
                    };
                    builder.append(4, format!("{}{} @{}", name, annotation, span));
                }
            }
            builder.append(2, "body");
            builder.nest(4, &function.body);
        }

        // Call (e) @1..2
        //   expr
        //     [expr]
        //   args
        //     [expr]
        Expression::Call { expr, args } => {
            builder.append(2, "expr");
            builder.nest(4, expr);
            if args.is_empty() {
                builder.append(2, "args: None");
            } else {
                builder.append(2, "args");
                for arg in args {
                    builder.nest(4, arg);
                }
            }
        }

        // MethodCall (e) @1..2
        //   expr
        //     [expr]
        //   name: [name] @1
        //   args
        //     [expr]
        Expression::MethodCall { expr, name, args } => {
            builder.append(2, "expr");
            builder.nest(4, expr);
            builder.append(2, format!("name: {} @{}", name.0, name.1));
            if args.is_empty() {
                builder.append(2, "args: None");
            } else {
                builder.append(2, "args");
                for arg in args {
                    builder.nest(4, arg);
                }
            }
        }

        // IndexAccess (e) @1..2
        //   expr
        //     [expr]
        //   accessor
        //     [expr]
        Expression::IndexAccess { expr, accessor } => {
            builder.append(2, "expr");
            builder.nest(4, expr);
            builder.append(2, "accessor");
            builder.nest(4, accessor);
        }

        // -- DotAccess (e) @1..2
        //   expr
        //     [expr]
        //   accessor: [name] @1
        Expression::DotAccess { expr, accessor } => {
            builder.append(2, "expr");
            builder.nest(4, expr);
            builder.append(2, format!("accessor: {} @{}", accessor.0, accessor.1));
        }

        // Error (e) @1..2
        Expression::Error => {}
    }
}
fn spanned_expression_pretty_print_inner(
    builder: &mut PrettyPrintBuilder,
    expr: &Expression<'_>,
    span: &TextSpan,
) {
    if let Expression::Primitive(_, expr_span) | Expression::Local(_, expr_span) = expr {
        debug_assert_eq!(span, expr_span);
        builder.append(0, expression_name(expr));
    } else {
        builder.append(0, format!("{} @{}", expression_name(expr), span));
    }
    expression_pretty_print_inner(builder, expr);
}
macro_rules! impl_pretty_print_for_spanned_expression {
    ($ty:ty) => {
        impl PrettyPrint for ($ty, TextSpan) {
            fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
                let (expr, span) = self;
                spanned_expression_pretty_print_inner(builder, expr, span);
            }
        }
    };
}
impl_pretty_print_for_spanned_expression!(Expression<'_>);
impl_pretty_print_for_spanned_expression!(&Expression<'_>);
impl_pretty_print_for_spanned_expression!(Box<Expression<'_>>);
impl PrettyPrint for Expression<'_> {
    fn pretty_print(&self, builder: &mut PrettyPrintBuilder) {
        builder.append(0, expression_name(self));
        expression_pretty_print_inner(builder, self);
    }
}
