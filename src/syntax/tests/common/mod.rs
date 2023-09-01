use syntax::tree::node::Block;
use syntax::{parse_src_to_token, parse_token_to_syntax_tree};

pub fn do_tree_test(src: &str, st: Block<'_>) {
    let tokens = parse_src_to_token(src);
    let res_st = parse_token_to_syntax_tree(&tokens, src.len()..src.len());
    assert_eq!(res_st, (st, (0..src.len()).into()));
}
