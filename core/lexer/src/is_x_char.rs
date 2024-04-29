use unicode_ident::{is_xid_continue, is_xid_start};
use unicode_properties::UnicodeEmoji;

/// Ref:
///   - https://github.com/rust-lang/rust/
///   - https://learn.microsoft.com/en-us/dotnet/api/system.char.iswhitespace?view=net-8.0
#[rustfmt::skip]
pub(crate) fn is_whitespace_char(c: char) -> bool {
    matches!(
        c,
        // U+0009: \t
        // U+000A: \n
        // U+000B: vertical tab
        // U+000C: form feed
        // U+000D: \r
        | '\u{0009}' ..= '\u{000D}'
        | '\u{0085}' // NEL (NEXT LINE from latin1)
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // =UnicodeCategory.SpaceSeparator:
        | '\u{0020}' // space
        | '\u{00A0}' // no-break space
        | '\u{1680}' // ogham space mark
        // U+2000: en quad
        // U+2001: em quad
        // U+2002: en space
        // U+2003: em space
        // U+2004: three-per-em space
        // U+2005: four-per-em space
        // U+2006: six-per-em space
        // U+2007: figure space
        // U+2008: punctuation space
        // U+2009: thin space
        // U+200A: hair space
        | '\u{2000}'..='\u{200A}'
        | '\u{202F}' // narrow no-break space
        | '\u{205F}' // medium mathematical space
        | '\u{3000}' // ideographic space

        // UnicodeCategory.LineSeparator category:
        | '\u{2028}'

        // UnicodeCategory.ParagraphSeparator category:
        | '\u{2029}'

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
