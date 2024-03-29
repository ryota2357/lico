use foundation::ast::Program;

pub fn parse_program(src: &str) -> Program<'_> {
    let (tokens, _) = lexer::parse(src);
    let (program, _) = parser::parse(&tokens);
    program
}
