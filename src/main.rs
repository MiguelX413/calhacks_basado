mod types;

use crate::types::defs::{
    Arrow, BinaryOperator, Comment, Delimiter, Dot, Keyword, ParseTokenError, Separator, Token,
    TokenKind,
};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct SplitTokens<'a> {
    remainder: &'a str,
    original: &'a str,
}

impl<'a> SplitTokens<'a> {
    pub fn new(string: &str) -> SplitTokens {
        SplitTokens {
            remainder: string,
            original: string,
        }
    }
}

macro_rules! sp {
    ($char:literal) => {
        ($char, _)
    };
    ($char1:literal, $char2:literal) => {
        ($char1, Some(($char2, _)))
    };
    ($char1:literal, $char2:literal, $char3:literal) => {
        ($char1, Some(($char2, Some($char3))))
    };
}

macro_rules! st {
    ($char:literal, $token_kind:expr, $remainder:expr) => {{
        const CHAR: char = $char;
        const LEN: usize = CHAR.len_utf8();
        let token_kind: crate::types::defs::TokenKind = $token_kind;
        let remainder: &str = $remainder;
        Some(Ok((
            crate::types::defs::Token::new(token_kind, &remainder[..LEN]),
            &remainder[LEN..],
        )))
    }};
    ($char1:literal, $char2:literal, $token_kind:expr, $remainder:expr) => {{
        const CHAR1: char = $char1;
        const CHAR2: char = $char2;
        const LEN: usize = CHAR1.len_utf8() + CHAR2.len_utf8();
        let token_kind: crate::types::defs::TokenKind = $token_kind;
        let remainder: &str = $remainder;
        Some(Ok((
            crate::types::defs::Token::new(token_kind, &remainder[..LEN]),
            &remainder[LEN..],
        )))
    }};
}

impl<'a> Iterator for SplitTokens<'a> {
    type Item = Result<Token<'a>, ParseTokenError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let trimmed = self.remainder.trim();

        if trimmed.is_empty() {
            return None;
        }

        let mut chars = trimmed.chars();
        match (chars.next()?, chars.next().map(|c| (c, chars.next()))) {
            ('0'..='9', _) | ('+' | '-', Some(('0'..='9', _))) => {
                let (token, remainder) = trimmed.split_at(
                    trimmed
                        .char_indices()
                        .skip(1)
                        .find(|(_, f)| !(f.is_ascii_digit()))
                        .map(|(i, _)| i)
                        .unwrap_or(trimmed.len()),
                );
                match token.parse() {
                    Ok(i) => Some(Ok((Token::new(TokenKind::Number(i), token), remainder))),
                    Err(e) => Some(Err(ParseTokenError::ParseIntError(e, token))),
                }
            }
            sp!('-', '>') => st!('-', '>', TokenKind::Arrow(Arrow::RArrow), trimmed),
            sp!('=', '>') => st!('=', '>', TokenKind::Arrow(Arrow::FatArrow), &trimmed),
            sp!('/', '/', '/') => {
                let (token, remainder) =
                    trimmed.split_at(trimmed.find('\n').unwrap_or(trimmed.len()));
                Some(Ok((
                    Token::new(TokenKind::Comment(Comment::DocComment), token),
                    remainder,
                )))
            }
            sp!('/', '/') => {
                let (token, remainder) =
                    trimmed.split_at(trimmed.find('\n').unwrap_or(trimmed.len()));
                Some(Ok((
                    Token::new(TokenKind::Comment(Comment::Comment), token),
                    remainder,
                )))
            }
            sp!('+') => st!('+', TokenKind::Operator(BinaryOperator::Add), trimmed),
            sp!('-') => st!('-', TokenKind::Operator(BinaryOperator::Sub), trimmed),
            sp!('*', '*') => st!('*', '*', TokenKind::Operator(BinaryOperator::Pow), trimmed),
            sp!('*') => st!('*', TokenKind::Operator(BinaryOperator::Mul), trimmed),
            sp!('/') => st!('/', TokenKind::Operator(BinaryOperator::Div), trimmed),
            sp!('%') => st!('%', TokenKind::Operator(BinaryOperator::Mod), trimmed),
            sp!(':', '=') => st!(
                ':',
                '=',
                TokenKind::Operator(BinaryOperator::Assign),
                trimmed
            ),
            sp!('=', '=') => st!('=', '=', TokenKind::Operator(BinaryOperator::Eq), trimmed),
            sp!('!', '=') => st!('!', '=', TokenKind::Operator(BinaryOperator::Ne), trimmed),
            sp!('>', '=') => st!('>', '=', TokenKind::Operator(BinaryOperator::Ge), trimmed),
            sp!('<', '=') => st!('<', '=', TokenKind::Operator(BinaryOperator::Le), trimmed),
            sp!('>') => st!('>', TokenKind::Operator(BinaryOperator::Gt), trimmed),
            sp!('<') => st!('<', TokenKind::Operator(BinaryOperator::Lt), trimmed),
            sp!('|') => st!('|', TokenKind::Operator(BinaryOperator::Or), trimmed),
            sp!('&') => st!('&', TokenKind::Operator(BinaryOperator::And), trimmed),
            sp!('{') => st!('{', TokenKind::Delimiter(Delimiter::CurlyLeft), trimmed),
            sp!('}') => st!('}', TokenKind::Delimiter(Delimiter::CurlyRight), trimmed),
            sp!('[') => st!('[', TokenKind::Delimiter(Delimiter::SquareLeft), trimmed),
            sp!(']') => st!(']', TokenKind::Delimiter(Delimiter::SquareRight), trimmed),
            sp!('(') => st!('(', TokenKind::Delimiter(Delimiter::ParLeft), trimmed),
            sp!(')') => st!(')', TokenKind::Delimiter(Delimiter::ParRight), trimmed),
            sp!(',') => st!(',', TokenKind::Separator(Separator::Comma), trimmed),
            sp!(':') => st!(':', TokenKind::Separator(Separator::Colon), trimmed),
            sp!(';') => st!(';', TokenKind::Separator(Separator::Semi), trimmed),
            sp!('.') => st!('.', TokenKind::Dot(Dot::Dot), trimmed),
            ('"', _) => {
                let mut escaped = false;
                let Some(index) = trimmed
                    .char_indices()
                    .skip(1)
                    .find(|(_, c)| match (c, escaped) {
                        ('\\', false) => {
                            escaped = true;
                            false
                        }
                        ('"', false) => true,
                        (_, true) => {
                            escaped = false;
                            false
                        }
                        (_, false) => false,
                    })
                    .map(|(i, _)| i)
                else {
                    return Some(Err(ParseTokenError::UnterminatedString));
                };
                let mut escaped = false;
                match trimmed[1..index]
                    .chars()
                    .filter_map(|c| match (c, escaped) {
                        ('\\', false) => {
                            escaped = true;
                            None
                        }
                        (cc, true) => {
                            escaped = false;
                            match cc {
                                '\\' => Some(Ok('\\')),
                                'n' => Some(Ok('\n')),
                                't' => Some(Ok('\t')),
                                '0' => Some(Ok('\0')),
                                '"' => Some(Ok('"')),
                                '\'' => Some(Ok('\'')),
                                ccc => Some(Err(ParseTokenError::InvalidEscape(ccc))),
                            }
                        }
                        (c, false) => Some(Ok(c)),
                    })
                    .collect::<Result<_, _>>()
                {
                    Ok(string) => Some(Ok((
                        Token::new(TokenKind::String(string), &trimmed[..=index]),
                        &trimmed[index + 1..],
                    ))),
                    Err(e) => Some(Err(e)),
                }
            }
            (c, _) if c.is_alphabetic() | (c == '_') => {
                let mut is_macro = false;
                let (token, remainder) = trimmed.split_at(
                    if c.is_uppercase() {
                        trimmed.find(|c: char| !(c.is_alphanumeric() | (c == '_')))
                    } else {
                        trimmed.find(|c| match (c, is_macro) {
                            (_, true) => true,
                            ('!', false) => {
                                is_macro = true;
                                false
                            }
                            (cc, _) => !(cc.is_alphanumeric() | (cc == '_')),
                        })
                    }
                    .unwrap_or(trimmed.len()),
                );
                Some(Ok((
                    Token::new(
                        match token {
                            "if" => TokenKind::Keyword(Keyword::If),
                            "else" => TokenKind::Keyword(Keyword::Else),
                            "match" => TokenKind::Keyword(Keyword::Match),
                            "while" => TokenKind::Keyword(Keyword::While),
                            "loop" => TokenKind::Keyword(Keyword::Loop),
                            "true" => TokenKind::Keyword(Keyword::True),
                            "false" => TokenKind::Keyword(Keyword::False),
                            "let" => TokenKind::Keyword(Keyword::Let),
                            "type" => TokenKind::Keyword(Keyword::Type),
                            "return" => TokenKind::Keyword(Keyword::Return),
                            "gen" => TokenKind::Keyword(Keyword::Gen),
                            "func" => TokenKind::Keyword(Keyword::Func),
                            _ if is_macro => TokenKind::MacroName,
                            _ if c.is_uppercase() => TokenKind::TypeName,
                            _ => TokenKind::Name,
                        },
                        token,
                    ),
                    remainder,
                )))
            }
            (c, _) => Some(Err(ParseTokenError::InvalidChar(
                c,
                &trimmed[..c.len_utf8()],
            ))),
        }
        .map(|f| {
            f.map(|(token, remainder)| {
                self.remainder = remainder;
                token
            })
        })
    }
}

pub fn split_tokens(string: &str) -> SplitTokens {
    SplitTokens::new(string)
}

pub fn main() {
    [
        "catfood-45",
        "catfood",
        "67z23",
        "catfood&-45",
        "&",
        " -45 - 45 + +45",
        "if +2 + -2 else x := x - 5 ",
        "if {{10 / {45 + 3}} + {2 * 4}} - +5",
        "日本語a+123",
        "cat- 32432432432432-ref",
        "{2133 ** 21} % 2",
        "let my_string := \"lol\\\"test\";
let xd := 2;",
    ]
    .into_iter()
    .for_each(|string| {
        println!(
            "{string:?}: {:?}",
            split_tokens(string).collect::<Result<Vec<_>, _>>()
        )
    });
}
