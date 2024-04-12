use unicode_ident::{is_xid_continue, is_xid_start};
use unicode_properties::UnicodeEmoji;

/// From rustc_lexer (https://github.com/rust-lang/rust/)
pub(crate) fn is_whitespace_char(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

pub(crate) fn is_ident_start_char(c: char) -> bool {
    c == '_' || is_xid_start(c)
}

pub(crate) fn is_ident_continue_char(c: char) -> bool {
    is_xid_continue(c)
}

pub(crate) fn is_emoji_char(c: char) -> bool {
    !c.is_ascii() && c.is_emoji_char()
}
