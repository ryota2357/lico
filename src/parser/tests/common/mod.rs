use parser::tree::Program;

pub fn parse_program(src: &str) -> Program<'_> {
    let (tokens, _) = lexer::parse(src);
    return parser::parse(&tokens, src.len()..src.len());
}
