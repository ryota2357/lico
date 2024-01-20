use super::*;
use unicode_ident::{is_xid_continue, is_xid_start};

pub fn tokenize(lexer: &mut Lexer) {
    loop {
        lexer.consume_ws();
        let Some(c) = lexer.next() else {
            break;
        };
        match c {
            '0'..='9' => tokenize_number(lexer, c),
            _ if is_xid_start(c) || c == '_' => tokenize_identifier(lexer),
            '"' => tokenize_string(lexer, '"'),
            '\'' => tokenize_string(lexer, '\''),
            '@' => tokenize_attribute(lexer),
            '#' => tokenize_comment(lexer),

            // operators
            '+' => match lexer.peek() {
                Some('+') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("++", span));
                    lexer.bump(Token::Error("++"));
                }
                _ => lexer.bump(Token::Plus),
            },
            '-' => match lexer.peek() {
                Some('>') => {
                    lexer.next();
                    lexer.bump(Token::Arrow);
                }
                Some('-') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("--", span));
                    lexer.bump(Token::Error("--"));
                }
                _ => lexer.bump(Token::Minus),
            },
            '*' => match lexer.peek() {
                Some('*') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("**", span));
                    lexer.bump(Token::Error("**"));
                }
                _ => lexer.bump(Token::Star),
            },
            '/' => match lexer.peek() {
                Some('/') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("//", span));
                    lexer.bump(Token::Error("//"));
                }
                _ => lexer.bump(Token::Slash),
            },
            '%' => lexer.bump(Token::Mod),
            '&' => match lexer.peek() {
                Some('&') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("&&", span));
                    lexer.bump(Token::Error("&&"));
                }
                _ => lexer.bump(Token::Amp),
            },
            '|' => match lexer.peek() {
                Some('|') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("||", span));
                    lexer.bump(Token::Error("||"));
                }
                Some('>') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("|>", span));
                    lexer.bump(Token::Error("|>"));
                }
                _ => lexer.bump(Token::Pipe),
            },
            '^' => lexer.bump(Token::Caret),
            '~' => match lexer.peek() {
                Some('=') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("~=", span));
                    lexer.bump(Token::Error("~="));
                }
                _ => lexer.bump(Token::Tilde),
            },
            '=' => match lexer.peek() {
                Some('=') => {
                    lexer.next();
                    lexer.bump(Token::Eq);
                }
                Some('>') => {
                    lexer.next();
                    lexer.report(|span| Error::UnsupportedOperator("=>", span));
                    lexer.bump(Token::Error("=>"));
                }
                _ => lexer.bump(Token::Assign),
            },
            '!' => match lexer.peek() {
                Some('=') => {
                    lexer.next();
                    lexer.bump(Token::NotEq);
                }
                _ => {
                    lexer.report(|span| Error::UnsupportedOperator("!", span));
                    lexer.bump(Token::Error("!"));
                }
            },
            '<' => match lexer.peek() {
                Some('=') => {
                    lexer.next();
                    lexer.bump(Token::LessEq);
                }
                Some('<') => {
                    lexer.next();
                    lexer.bump(Token::Less2);
                }
                _ => lexer.bump(Token::Less),
            },
            '>' => match lexer.peek() {
                Some('=') => {
                    lexer.next();
                    lexer.bump(Token::GreaterEq);
                }
                Some('>') => {
                    lexer.next();
                    lexer.bump(Token::Greater2);
                }
                _ => lexer.bump(Token::Greater),
            },
            '.' => match lexer.peek() {
                Some('.') => {
                    lexer.next();
                    lexer.bump(Token::Dot2);
                }
                _ => lexer.bump(Token::Dot),
            },

            // delimiters
            ',' => lexer.bump(Token::Comma),
            ':' => lexer.bump(Token::Colon),
            '(' => lexer.bump(Token::OpenParen),
            ')' => lexer.bump(Token::CloseParen),
            '{' => lexer.bump(Token::OpenBrace),
            '}' => lexer.bump(Token::CloseBrace),
            '[' => lexer.bump(Token::OpenBracket),
            ']' => lexer.bump(Token::CloseBracket),

            _ => {
                lexer.take_until(|c| c.is_whitespace());
                let slice = lexer.get_slice();
                lexer.report(|span| Error::InvalidInputSequence(slice.into(), span));
                lexer.bump(Token::Error(slice));
            }
        }
    }
}

fn tokenize_number(lexer: &mut Lexer, start: char) {
    let radix = if start == '0' {
        match lexer.peek() {
            Some('b') => {
                lexer.next();
                2
            }
            Some('o') => {
                lexer.next();
                8
            }
            Some('x') => {
                lexer.next();
                16
            }
            Some('0'..='9' | '.') => 10,
            Some(c) if is_xid_start(c) || c == '_' => {
                lexer.take_until(|c| c.is_whitespace());
                let slice = lexer.get_slice();
                lexer.report(|span| Error::UnknownNumberLiteral(slice.into(), span));
                lexer.bump(Token::Error(slice));
                return;
            }
            Some(_) => {
                lexer.bump(Token::Int(0));
                return;
            }
            None => {
                lexer.bump(Token::Int(0));
                return;
            }
        }
    } else {
        10
    };
    lexer.take_while(|c| c.is_digit(radix));
    if lexer.peek() == Some('.') {
        lexer.next();
        if radix != 10 {
            lexer.take_while(|c| c.is_digit(radix));
            let slice = lexer.get_slice();
            lexer.report(|span| Error::InvalidFloatLiteral {
                info: (slice.into(), span),
                reason: format!(
                    "{} float literal is not supported",
                    match radix {
                        2 => "Binary",
                        8 => "Octal",
                        16 => "Hexadecimal",
                        _ => unreachable!(),
                    }
                ),
            });
        } else {
            lexer.take_while(|c| c.is_ascii_digit()); // U+0030 '0' ..= U+0039 '9'
            let slice = lexer.get_slice();
            match slice.parse() {
                Ok(x) => lexer.bump(Token::Float(x)),
                Err(err) => {
                    lexer.report(|span| Error::InvalidFloatLiteral {
                        info: (slice.into(), span),
                        reason: format!("Invalid float literal: {}", err),
                    });
                }
            }
        }
    } else {
        let slice = if radix == 10 {
            lexer.get_slice()
        } else {
            &lexer.get_slice()[2..] // skip '0b', '0o', '0x'
        };
        match i64::from_str_radix(slice, radix) {
            Ok(x) => lexer.bump(Token::Int(x)),
            Err(err) => {
                lexer.report(|span| Error::InvalidIntLiteral {
                    info: (slice.into(), span),
                    source: err,
                });
            }
        }
    }
}

fn tokenize_identifier(lexer: &mut Lexer<'_>) {
    lexer.take_while(is_xid_continue);
    let slice = lexer.get_slice();
    let token = match slice {
        "var" => Token::Var,
        "func" => Token::Func,
        "if" => Token::If,
        "then" => Token::Then,
        "elif" => Token::Elif,
        "else" => Token::Else,
        "for" => Token::For,
        "while" => Token::While,
        "in" => Token::In,
        "ref" => Token::Ref,
        "do" => Token::Do,
        "end" => Token::End,
        "return" => Token::Return,
        "break" => Token::Break,
        "continue" => Token::Continue,
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "nil" => Token::Nil,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        _ => Token::Ident(slice),
    };
    lexer.bump(token);
}

fn tokenize_string(lexer: &mut Lexer<'_>, start: char) {
    let mut content = String::new();

    fn bump_string(lexer: &mut Lexer<'_>, content: String, skip_last: usize) {
        if content.is_empty() {
            let slice = &lexer.get_slice()[1..]; // skip first quote
            let content = &slice[..slice.len() - skip_last];
            lexer.bump(Token::String(content.into()));
        } else {
            lexer.bump(Token::String(content.into()));
        }
    }

    // NOTE: `last` should be be `'`, `"` or `None`
    fn report_missing_closing_delimiter(lexer: &mut Lexer, delimiter: char, last: Option<char>) {
        let last_len = if last.is_some() { 1 } else { 0 };
        lexer.report(|span| Error::MissingClosingDelimiter {
            info: (last, TextSpan::at(span.end() - last_len, last_len)),
            expected: delimiter,
        });
    }

    loop {
        match lexer.next() {
            Some(c) if c == start => {
                bump_string(lexer, content, 1); // 1 == last quote
                return;
            }
            c @ (None | Some('\n' | '\r')) => {
                lexer.report(|span| Error::MissingClosingDelimiter {
                    info: (c, TextSpan::at(span.end() - 1, 1)),
                    expected: start,
                });
                // NOTE: `c` is one of ['\n', '\r', None]
                bump_string(lexer, content, if c.is_some() { 1 } else { 0 });
                return;
            }
            Some('\\') => {
                if content.is_empty() {
                    let slice = lexer.get_slice();
                    content.push_str(&slice[1..slice.len() - 1]) // skip first quote and last backslash
                }
                match lexer.next() {
                    Some('\\') => content.push('\\'),
                    Some('n') => content.push('\n'),
                    Some('r') => content.push('\r'),
                    Some('t') => content.push('\t'),
                    Some('0') => content.push('\0'),
                    Some('\'') => content.push('\''),
                    Some('"') => content.push('"'),
                    Some('x') => {
                        let o = match lexer.next() {
                            Some(c @ '0'..='7') => {
                                // SAFETY: `c` is ensured to be [0-7] by the match guard
                                unsafe { c.to_digit(8).unwrap_unchecked() }
                            }
                            c @ (None | Some('\n' | '\r')) => {
                                report_missing_closing_delimiter(lexer, start, c);
                                lexer.bump(Token::String((content + r"\x").into()));
                                return;
                            }
                            Some(c) if c == start => {
                                lexer.report(|span| Error::InvalidEscapeSequence {
                                    info: (r"\x".to_string(), TextSpan::at(span.end() - 2, 2)),
                                    reason: r"`\x` escape sequence requires 2 hex digits".into(),
                                });
                                lexer.bump(Token::String((content + r"\x").into()));
                                return;
                            }
                            Some(c) => {
                                lexer.report(|span| {
                                    let c_len = c.len_utf8() as u32;
                                    Error::UnexpectedCharInEscapeSequence {
                                        info: (c, TextSpan::at(span.end() - c_len, c_len)),
                                        expected: vec![('0'..='7')],
                                    }
                                });
                                content.push_str(format!(r"\x{}", c).as_str());
                                continue;
                            }
                        };
                        let x = match lexer.next() {
                            Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) => {
                                // SAFETY: `c` is ensured to be [0-9a-fA-F] by the match guard
                                unsafe { c.to_digit(16).unwrap_unchecked() }
                            }
                            c @ (None | Some('\n' | '\r')) => {
                                report_missing_closing_delimiter(lexer, start, c);
                                lexer.bump(Token::String((content + &format!(r"\x{}", o)).into()));
                                return;
                            }
                            Some(c) if c == start => {
                                lexer.report(|span| Error::InvalidEscapeSequence {
                                    info: (format!(r"\x{}", o), TextSpan::at(span.end() - 3, 3)),
                                    reason: r"`\x` escape sequence requires 2 hex digits".into(),
                                });
                                lexer.bump(Token::String(content.into()));
                                return;
                            }
                            Some(c) => {
                                lexer.report(|span| {
                                    let c_len = c.len_utf8() as u32;
                                    Error::UnexpectedCharInEscapeSequence {
                                        info: (c, TextSpan::at(span.end() - c_len, c_len)),
                                        expected: vec![('0'..='9'), ('a'..='f'), ('A'..='F')],
                                    }
                                });
                                content.push_str(format!(r"\x{}{}", o, c).as_str());
                                continue;
                            }
                        };
                        let c = std::char::from_u32(o * 16 + x).unwrap();
                        content.push(c);
                    }
                    Some('u') => {
                        match lexer.next() {
                            Some('{') => { /* only consume `{` */ }
                            c @ (None | Some('\n' | '\r')) => {
                                report_missing_closing_delimiter(lexer, start, c);
                                lexer.bump(Token::String((content + r"\u").into()));
                                return;
                            }
                            Some(c) if c == start => {
                                lexer.report(|span| Error::InvalidEscapeSequence {
                                    info: (r"\u{}".to_string(), TextSpan::at(span.end() - 2, 2)),
                                    reason: r"`\u` escape sequence format is `\u{...}`".into(),
                                });
                                lexer.bump(Token::String(content.into()));
                                return;
                            }
                            Some(c) => {
                                lexer.report(|span| {
                                    let c_len = c.len_utf8() as u32;
                                    Error::UnexpectedCharInEscapeSequence {
                                        info: (c, TextSpan::at(span.end() - c_len, c_len)),
                                        expected: vec![('{'..='{')],
                                    }
                                });
                                content.push_str(r"\u");
                                continue;
                            }
                        }
                        let mut codepoint = String::new();
                        loop {
                            match lexer.next() {
                                Some('}') => {
                                    let char = std::char::from_u32(
                                        // `codepoint` should be valid hex
                                        u32::from_str_radix(&codepoint, 16).unwrap(),
                                    );
                                    if let Some(char) = char {
                                        content.push(char);
                                    } else {
                                        let esc_string = r"\u{".to_owned() + &codepoint + "}";
                                        lexer.report(|span| {
                                            let esc_len = esc_string.len() as u32;
                                            let span = TextSpan::at(span.end() - esc_len, esc_len);
                                            Error::InvalidEscapeSequence {
                                                info: (esc_string, span),
                                                reason: "invalid unicode codepoint".into(),
                                            }
                                        });
                                        content.push_str(r"\u{");
                                        content.push_str(&codepoint);
                                        content.push('}');
                                    }
                                    break;
                                }
                                Some(c) if c.is_ascii_hexdigit() => {
                                    codepoint.push(c);
                                    continue;
                                }
                                Some(c) if c == start => {
                                    let esc_string = r"\u{".to_owned() + &codepoint;
                                    lexer.report(|span| {
                                        let esc_len = esc_string.len() as u32;
                                        let span = TextSpan::at(span.end() - esc_len, esc_len);
                                        Error::InvalidEscapeSequence {
                                            info: (esc_string, span),
                                            reason: r"`\u` escape sequence format is `\u{...}`"
                                                .into(),
                                        }
                                    });
                                    lexer.bump(Token::String(content.into()));
                                    return;
                                }
                                c @ (None | Some('\n' | '\r')) => {
                                    report_missing_closing_delimiter(lexer, start, c);
                                    let string = content + r"\u{" + &codepoint;
                                    lexer.bump(Token::String(string.into()));
                                    return;
                                }
                                Some(c) => {
                                    lexer.report(|span| {
                                        let c_len = c.len_utf8() as u32;
                                        Error::UnexpectedCharInEscapeSequence {
                                            info: (c, TextSpan::at(span.end() - c_len, c_len)),
                                            expected: vec![('0'..='9'), ('a'..='f'), ('A'..='F')],
                                        }
                                    });
                                    content.push_str(r"\u{");
                                    content.push_str(&codepoint);
                                    break;
                                }
                            }
                        }
                    }
                    c @ (None | Some('\n' | '\r')) => {
                        report_missing_closing_delimiter(lexer, start, c);
                        lexer.bump(Token::String(content.into()));
                        return;
                    }
                    Some(c) => {
                        lexer.report(|span| {
                            let c_len = c.len_utf8() as u32;
                            Error::InvalidEscapeSequence {
                                info: (format!(r"\{}", c), TextSpan::at(span.end() - c_len, c_len)),
                                reason: "unknown escape sequence".into(),
                            }
                        });
                        content.push('\\');
                        content.push(c);
                    }
                }
            }
            Some(c) => {
                if !content.is_empty() {
                    content.push(c);
                }
            }
        }
    }
}

fn tokenize_attribute(lexer: &mut Lexer<'_>) {
    // NOTE: is_ascii_alphanumeric() == a..=z | A..=Z | 0..=9
    lexer.take_while(|c| c.is_ascii_alphanumeric() || c == '_');
    let slice = &lexer.get_slice()[1..]; // skip '@'
    lexer.bump(Token::Attribute(slice));
}

fn tokenize_comment(lexer: &mut Lexer<'_>) {
    loop {
        let Some(c) = lexer.next() else {
            let slice = &lexer.get_slice()[1..]; // skip '#'
            lexer.bump(Token::Comment(slice));
            return;
        };
        if ['\n', '\r'].contains(&c) {
            break;
        }
    }
    let slice = &lexer.get_slice()[1..]; // skip '#'
    let line = &slice[..slice.len() - 1]; // skip last newline
    lexer.bump(Token::Comment(line));
}
