use parser::tree::{Chunk, Program};

pub fn do_chunk_test(src: &str, chunk: Chunk<'_>) {
    let tokens = lexer::parse(src);
    let tree = parser::parse(&tokens, src.len()..src.len());
    let res = tree.body;
    assert_eq!(res, chunk);
}

#[allow(dead_code)]
pub fn do_teset(src: &str, tree: Program<'_>) {
    let tokens = lexer::parse(src);
    let res = parser::parse(&tokens, src.len()..src.len());
    assert_eq!(res, tree);
}
