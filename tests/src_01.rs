use assert2::*;

use texfmt::lexer::lex_tokens;

#[test]
fn tokenize_all() {
    let src = include_str!("../assets/src_01.tex");
    let result = lex_tokens(src);
    let_assert!(Ok((rest, tokens)) = result);
    check!(rest.is_empty());
    check!(!tokens.is_empty());
    check!(tokens.len() == 286);
}
