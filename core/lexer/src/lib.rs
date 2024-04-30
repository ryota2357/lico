mod cursor;
use cursor::Cursor;

mod is_x_char;
use is_x_char::*;

use foundation::syntax::token::{TokenKind::*, *};

pub fn tokenize(source: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(source);
    core::iter::from_fn(move || advance_token(&mut cursor))
}

fn advance_token(cursor: &mut Cursor) -> Option<Token> {
    let kind = match cursor.next()? {
        c if is_whitespace_char(c) => whitespace(cursor),
        c if is_ident_start_char(c) => ident_or_keyword(cursor, c),
        c @ '0'..='9' => number(cursor, c),
        c @ ('"' | '\'') => string(cursor, c),
        '#' => comment(cursor),
        '+' => Plus,
        '-' => symbol2(Minus, cursor, [('>', Arrow)]),
        '*' => Star,
        '/' => Slash,
        '%' => Percent,
        '&' => Amp,
        '|' => Pipe,
        '^' => Caret,
        '~' => Tilde,
        '!' => symbol2(Bang, cursor, [('=', BangEq)]),
        '=' => symbol2(Eq, cursor, [('=', Eq2)]),
        '<' => symbol2(Lt, cursor, [('=', LtEq), ('<', Lt2)]),
        '>' => symbol2(Gt, cursor, [('=', GtEq), ('>', Gt2)]),
        '.' => symbol2(Dot, cursor, [('.', Dot2)]),
        '@' => At,
        ',' => Comma,
        ':' => Colon,
        '(' => OpenParen,
        ')' => CloseParen,
        '{' => OpenBrace,
        '}' => CloseBrace,
        '[' => OpenBracket,
        ']' => CloseBracket,
        c if is_emoji_char(c) => invalid_ident(cursor),
        _ => Unknown,
    };
    let token = cursor.bump(kind);
    Some(token)
}

fn whitespace(cursor: &mut Cursor) -> TokenKind {
    debug_assert!(is_whitespace_char(cursor.prev()));
    cursor.eat_while(is_whitespace_char);
    Whitespace
}

fn ident_or_keyword(cursor: &mut Cursor, first_char: char) -> TokenKind {
    debug_assert!(is_ident_start_char(cursor.prev()));

    fn keyword_trie_tree(cursor: &mut Cursor, first_char: char) -> Option<TokenKind> {
        fn next_if_s(cursor: &mut Cursor, s: &[char], kind: TokenKind) -> Option<TokenKind> {
            for c in s {
                if cursor.peek()? == *c {
                    cursor.next();
                } else {
                    return None;
                }
            }
            Some(kind)
        }

        fn next_if_c(cursor: &mut Cursor, c: char, kind: TokenKind) -> Option<TokenKind> {
            if cursor.peek()? == c {
                cursor.next();
                Some(kind)
            } else {
                None
            }
        }

        let pre_match = match first_char {
            'a' => next_if_s(cursor, &['n', 'd'], And),
            'b' => next_if_s(cursor, &['r', 'e', 'a', 'k'], Break),
            'c' => next_if_s(cursor, &['o', 'n', 't', 'i', 'n', 'u', 'e'], Continue),
            'd' => next_if_c(cursor, 'o', Do),
            'e' => match cursor.peek()? {
                'l' => {
                    cursor.next();
                    match cursor.peek()? {
                        'i' => {
                            cursor.next();
                            next_if_c(cursor, 'f', Elif)
                        }
                        's' => {
                            cursor.next();
                            next_if_c(cursor, 'e', Else)
                        }
                        _ => None,
                    }
                }
                'n' => {
                    cursor.next();
                    next_if_c(cursor, 'd', End)
                }
                _ => None,
            },
            'f' => match cursor.peek()? {
                'a' => {
                    cursor.next();
                    next_if_s(cursor, &['l', 's', 'e'], False)
                }
                'o' => {
                    cursor.next();
                    next_if_c(cursor, 'r', For)
                }
                'u' => {
                    cursor.next();
                    next_if_s(cursor, &['n', 'c'], Func)
                }
                _ => None,
            },
            'i' => match cursor.peek()? {
                'f' => {
                    cursor.next();
                    Some(If)
                }
                'n' => {
                    cursor.next();
                    Some(In)
                }
                _ => None,
            },
            'n' => match cursor.peek()? {
                'i' => {
                    cursor.next();
                    next_if_c(cursor, 'l', Nil)
                }
                'o' => {
                    cursor.next();
                    next_if_c(cursor, 't', Not)
                }
                _ => None,
            },
            'o' => next_if_c(cursor, 'r', Or),
            'r' => next_if_s(cursor, &['e', 't', 'u', 'r', 'n'], Return),
            't' => match cursor.peek()? {
                'h' => {
                    cursor.next();
                    next_if_s(cursor, &['e', 'n'], Then)
                }
                'r' => {
                    cursor.next();
                    next_if_s(cursor, &['u', 'e'], True)
                }
                _ => None,
            },
            'v' => next_if_s(cursor, &['a', 'r'], Var),
            'w' => next_if_s(cursor, &['h', 'i', 'l', 'e'], While),
            _ => None,
        };
        if pre_match.is_some() {
            let Some(c) = cursor.peek() else {
                return pre_match;
            };
            if is_ident_continue_char(c) {
                None
            } else {
                pre_match
            }
        } else {
            None
        }
    }
    keyword_trie_tree(cursor, first_char).unwrap_or_else(|| {
        cursor.eat_while(is_ident_continue_char);
        Ident
    })
}

fn number(cursor: &mut Cursor, first_digit: char) -> TokenKind {
    debug_assert!(first_digit.is_ascii_digit()); // 0..=9

    fn eat_decimal_digits(cursor: &mut Cursor) -> bool {
        let mut has_digits = false;
        while let Some(c) = cursor.peek() {
            match c {
                '_' => {
                    cursor.next();
                }
                '0'..='9' => {
                    has_digits = true;
                    cursor.next();
                }
                _ => break,
            }
        }
        has_digits
    }
    fn eat_hexadecimal_digits(cursor: &mut Cursor) -> bool {
        let mut has_digits = false;
        while let Some(c) = cursor.peek() {
            match c {
                '_' => {
                    cursor.next();
                }
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    cursor.next();
                }
                _ => break,
            }
        }
        has_digits
    }

    let mut base = NumBase::Decimal;
    if first_digit == '0' {
        match cursor.peek() {
            Some('b') => {
                base = NumBase::Binary;
                cursor.next();
                if !eat_decimal_digits(cursor) {
                    return Int {
                        base,
                        empty_int: true,
                    };
                }
            }
            Some('o') => {
                base = NumBase::Octal;
                cursor.next();
                if !eat_decimal_digits(cursor) {
                    return Int {
                        base,
                        empty_int: true,
                    };
                }
            }
            Some('x') => {
                base = NumBase::Hexadecimal;
                cursor.next();
                if !eat_hexadecimal_digits(cursor) {
                    return Int {
                        base,
                        empty_int: true,
                    };
                }
            }
            Some('0'..='9' | '_') => {
                eat_decimal_digits(cursor);
            }
            Some('.' | 'e' | 'E') => {}
            _ => {
                // "0"
                return Int {
                    base,
                    empty_int: false,
                };
            }
        }
    } else {
        eat_decimal_digits(cursor);
    }

    fn eat_float_exponent(cursor: &mut Cursor) -> bool {
        assert!(matches!(cursor.next(), Some('e' | 'E')));
        if let Some('-') | Some('+') = cursor.peek() {
            cursor.next();
        }
        eat_decimal_digits(cursor)
    }

    match cursor.peek() {
        Some('.') => {
            cursor.next();
            let empty_exponent = match cursor.peek() {
                Some('0'..='9') => {
                    eat_decimal_digits(cursor);
                    match cursor.peek() {
                        Some('e' | 'E') => !eat_float_exponent(cursor),
                        _ => false,
                    }
                }
                _ => true,
            };
            Float {
                base,
                empty_exponent,
            }
        }
        Some('e' | 'E') => Float {
            base,
            empty_exponent: !eat_float_exponent(cursor),
        },
        _ => Int {
            base,
            empty_int: false,
        },
    }
}

fn string(cursor: &mut Cursor, quote: char) -> TokenKind {
    debug_assert!((quote == '"' || quote == '\'') && cursor.prev() == quote);
    let mut terminated = false;
    while let Some(c) = cursor.next() {
        match c {
            '\\' => {
                let peek = cursor.peek();
                if peek == Some(quote) || peek == Some('\\') {
                    cursor.next();
                }
            }
            q if q == quote => {
                terminated = true;
                break;
            }
            _ => {}
        }
    }
    String {
        terminated,
        quote_kind: match quote {
            '"' => QuoteKind::Double,
            '\'' => QuoteKind::Single,
            _ => unreachable!(),
        },
    }
}

fn comment(cursor: &mut Cursor) -> TokenKind {
    debug_assert!(cursor.prev() == '#');
    cursor.eat_while(|c| c != '\n');
    LineComment
}

fn symbol2<const N: usize>(
    default: TokenKind,
    cursor: &mut Cursor,
    rule: [(char, TokenKind); N],
) -> TokenKind {
    if let Some(peek) = cursor.peek() {
        for (c, kind) in rule {
            if c == peek {
                cursor.next();
                return kind;
            }
        }
    }
    default
}

fn invalid_ident(cursor: &mut Cursor) -> TokenKind {
    debug_assert!(is_emoji_char(cursor.prev()));
    cursor.eat_while(|c| is_ident_continue_char(c) || is_emoji_char(c) || c == '\u{200D}');
    InvalidIdent
}
