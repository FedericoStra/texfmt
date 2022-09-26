//! Tokenize a (La)TeX source.
//!
//! This module provides functions to perform lexical analysis of a (La)TeX source.
//!
//! # Examples
//!
//! ```rust
//! use texfmt::lexer::{lex_tokens, Token};
//! assert_eq!(
//!     lex_tokens(r"\cmd{arg}"),
//!     Ok((
//!         "",
//!         vec![Token::Command("cmd"), Token::LBrace, Token::Text("arg"), Token::RBrace]
//!     ))
//! );
//! ```

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, line_ending, none_of, not_line_ending, one_of, space1},
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::preceded,
    IResult,
};

/// (La)TeX tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token<S> {
    /// `\command`
    Command(S),
    /// `% comment`
    Comment(S),
    /// Regular text (none of the other tokens).
    Text(S),
    /// `\\`
    Endline,

    // math
    /// `\[`
    BDisplayMath,
    /// `\]`
    EDisplayMath,
    /// `$$`
    TDisplayMath,
    /// `$`
    InlineMath,

    // space
    /// `' '` or `'\t'`
    Whitespace(S),
    /// `'\n'` or `"\r\n"`
    Newline,

    // delimiters
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
}

type LexResult<'a> = IResult<&'a str, Token<&'a str>>;

fn lex_command(input: &str) -> LexResult<'_> {
    map(preceded(char('\\'), alpha1), Token::Command)(input)
}

fn lex_comment(input: &str) -> LexResult<'_> {
    map(preceded(char('%'), not_line_ending), Token::Comment)(input)
}

fn lex_math(input: &str) -> LexResult<'_> {
    alt((
        map(tag(r"\["), |_| Token::BDisplayMath),
        map(tag(r"\]"), |_| Token::EDisplayMath),
        map(tag(r"$$"), |_| Token::TDisplayMath),
        map(tag(r"$"), |_| Token::InlineMath),
    ))(input)
}

fn lex_endline(input: &str) -> LexResult<'_> {
    map(tag(r"\\"), |_| Token::Endline)(input)
}

// space

fn lex_whitespace(input: &str) -> LexResult<'_> {
    map(space1, Token::Whitespace)(input)
}

fn lex_newline(input: &str) -> LexResult<'_> {
    map(line_ending, |_| Token::Newline)(input)
}

// delimiters

fn lex_delimiter(input: &str) -> LexResult<'_> {
    alt((
        map(char('{'), |_| Token::LBrace),
        map(char('}'), |_| Token::RBrace),
        map(char('['), |_| Token::LBracket),
        map(char(']'), |_| Token::RBracket),
    ))(input)
}

// text

fn lex_text(input: &str) -> LexResult<'_> {
    map(
        // verify(
        //     escaped(none_of("\\%{}$ \t\n"), '\\', one_of("%{}$&,;! ")),
        //     |s: &str| !s.is_empty(),
        // ),
        recognize(many1(alt((
            none_of("\\%{}$ \t\n"),
            preceded(char('\\'), one_of("%{}$&,;! ")),
        )))),
        Token::Text,
    )(input)
}

/// Identiy the first token in the input (La)TeX string.
pub fn lex_token(input: &str) -> LexResult<'_> {
    alt((
        lex_command,
        lex_comment,
        lex_endline,
        lex_math,
        lex_whitespace,
        lex_newline,
        lex_delimiter,
        lex_text,
    ))(input)
}

/// Tokenize the input (La)TeX string.
pub fn lex_tokens(input: &str) -> IResult<&str, Vec<Token<&str>>> {
    many0(lex_token)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command() {
        assert_eq!(
            lex_command(r"\cmd{arg}"),
            Ok(("{arg}", Token::Command("cmd")))
        );

        assert_eq!(
            lex_command(r"\\cmd"),
            Err(nom::Err::Error(nom::error::Error {
                input: r"\cmd",
                code: nom::error::ErrorKind::Alpha
            }))
        );
    }

    #[test]
    fn math() {
        assert_eq!(lex_math(r"\[1+2\]"), Ok((r"1+2\]", Token::BDisplayMath)));
        assert_eq!(lex_math(r"\]asd"), Ok((r"asd", Token::EDisplayMath)));
        assert_eq!(lex_math(r"$$1+2$$"), Ok((r"1+2$$", Token::TDisplayMath)));
        assert_eq!(lex_math(r"$1+2$"), Ok((r"1+2$", Token::InlineMath)));
    }

    #[test]
    fn comment() {
        assert_eq!(
            lex_comment("% hello world"),
            Ok(("", Token::Comment(" hello world")))
        );
        assert_eq!(
            lex_comment("% hello world\n"),
            Ok(("\n", Token::Comment(" hello world")))
        );
    }

    #[test]
    fn text() {
        assert_eq!(lex_text("asd$"), Ok(("$", Token::Text("asd"))));
        assert_eq!(lex_text("1+2$"), Ok(("$", Token::Text("1+2"))));
        assert_eq!(
            lex_text(r"(\,\&\;)\["),
            Ok((r"\[", Token::Text(r"(\,\&\;)")))
        );
    }

    #[test]
    fn tokens_0() {
        assert_eq!(
            lex_tokens(r"\[1+2\]"),
            Ok((
                "",
                vec![Token::BDisplayMath, Token::Text("1+2"), Token::EDisplayMath]
            ))
        );
    }

    #[test]
    fn tokens_1() {
        assert_eq!(
            lex_tokens("\\cmd{arg} some\ttext \\{\\%\\} \t\n \\end % comment"),
            Ok((
                "",
                vec![
                    Token::Command("cmd"),
                    Token::LBrace,
                    Token::Text("arg"),
                    Token::RBrace,
                    Token::Whitespace(" "),
                    Token::Text("some"),
                    Token::Whitespace("\t"),
                    Token::Text("text"),
                    Token::Whitespace(" "),
                    Token::Text("\\{\\%\\}"),
                    Token::Whitespace(" \t"),
                    Token::Newline,
                    Token::Whitespace(" "),
                    Token::Command("end"),
                    Token::Whitespace(" "),
                    Token::Comment(" comment"),
                ]
            ))
        );
    }

    #[test]
    fn tokens_2() {
        assert_eq!(
            lex_tokens(r"\cmd{arg}$1+2$\\$$1+2$$ \[1+2\]"),
            Ok((
                "",
                vec![
                    Token::Command("cmd"),
                    Token::LBrace,
                    Token::Text("arg"),
                    Token::RBrace,
                    Token::InlineMath,
                    Token::Text("1+2"),
                    Token::InlineMath,
                    Token::Endline,
                    Token::TDisplayMath,
                    Token::Text("1+2"),
                    Token::TDisplayMath,
                    Token::Whitespace(" "),
                    Token::BDisplayMath,
                    Token::Text("1+2"),
                    Token::EDisplayMath,
                ]
            ))
        );
    }
}
