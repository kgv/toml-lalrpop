use super::{Error, Mode, Result, Span};
use std::{char, str::CharIndices};

pub fn unescape<'a>(input: &'a str, mode: Mode) -> impl 'a + Iterator<Item = Result<char>> {
    Unescape::new(input, mode).map(|r| r.map(|(_, c)| c))
}

/// Unescape.
#[derive(Clone)]
pub struct Unescape<'a> {
    char_indices: CharIndices<'a>,
    mode: Mode,
    span: Span,
}

impl<'a> Unescape<'a> {
    pub fn new(input: &'a str, mode: Mode) -> Unescape<'a> {
        Self {
            char_indices: input.char_indices(),
            mode,
            span: Span::new(),
        }
    }

    fn parse_escape(&mut self) -> Result<(Span, char)> {
        let (i, c) = self
            .char_indices
            .next()
            .ok_or(Error::LoneSlash(self.span))?;
        self.span.end = i + 1;
        match c {
            't' => Ok((self.span, '\t')),
            'n' => Ok((self.span, '\n')),
            'r' => Ok((self.span, '\r')),
            '"' => Ok((self.span, '"')),
            '\\' => Ok((self.span, '\\')),
            'u' => self.parse_unicode_escape(4),
            'U' => self.parse_unicode_escape(8),
            _ => return Err(Error::InvalidEscape(self.span)),
        }
    }

    fn parse_unicode_escape(&mut self, n: usize) -> Result<(Span, char)> {
        let mut value = 0;
        for _ in 0..n {
            let (i, c) = self
                .char_indices
                .next()
                .ok_or(Error::IncompleteUnicodeEscape(self.span))?;
            self.span.end = i + 1;
            let digit = c
                .to_digit(16)
                .ok_or(Error::InvalidCharInUnicodeEscape(self.span))?;
            value = value * 16 + digit;
        }
        char::from_u32(value).map_or_else(
            || match value {
                value if value > 0x10ffff => Err(Error::OutOfRangeUnicodeEscape(self.span)),
                _ => Err(Error::SurrogateUnicodeEscape(self.span)),
            },
            |c| Ok((self.span, c)),
        )
    }

    fn skip_ascii_whitespace(&mut self) {
        let str = self.char_indices.as_str();
        let first_non_space = str
            .bytes()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(str.len());
        self.char_indices = str[first_non_space..].char_indices()
    }
}

impl Iterator for Unescape<'_> {
    type Item = Result<(Span, char)>;

    fn next(&mut self) -> Option<Result<(Span, char)>> {
        loop {
            let (i, c) = self.char_indices.next()?;
            self.span = Span {
                start: i,
                end: i + 1,
            };
            return match c {
                '\t' => Some(Ok((self.span, c))),
                '\n' | '\r' if self.mode == Mode::MultiLine => Some(Ok((self.span, c))),
                c if c.is_ascii_control() => Some(Err(Error::EscapeOnlyChar(self.span))),
                '"' if self.mode == Mode::SingleLine => Some(Err(Error::EscapeOnlyChar(self.span))),
                '\\' => {
                    // Toml specification requires us to skip whitespaces if
                    // unescaped '\' character is followed by '\n'. For details
                    // see [TOML](https://github.com/toml-lang/toml#string).
                    if self.mode == Mode::MultiLine {
                        let mut attempt = self.char_indices.clone();
                        if let Some((_, '\n')) = attempt.next() {
                            self.skip_ascii_whitespace();
                            continue;
                        }
                    }
                    Some(self.parse_escape())
                }
                _ => Some(Ok((self.span, c))),
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ht() -> Result<()> {
        assert_eq!(
            unescape("a\tb", Mode::SingleLine).collect::<Result<String>>()?,
            "a\tb",
        );
        assert_eq!(
            unescape("a\tb", Mode::MultiLine).collect::<Result<String>>()?,
            "a\tb",
        );
        Ok(())
    }

    #[test]
    fn lf() -> Result<()> {
        assert_eq!(
            unescape("a\nb", Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::EscapeOnlyChar(Span { start: 1, end: 2 })),
        );
        assert_eq!(
            unescape("a\nb", Mode::MultiLine).collect::<Result<String>>()?,
            "a\nb",
        );
        Ok(())
    }

    #[test]
    fn cr() -> Result<()> {
        assert_eq!(
            unescape("a\rb", Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::EscapeOnlyChar(Span { start: 1, end: 2 })),
        );
        assert_eq!(
            unescape("a\rb", Mode::MultiLine).collect::<Result<String>>()?,
            "a\rb",
        );
        Ok(())
    }

    #[test]
    fn cr_lf() -> Result<()> {
        assert_eq!(
            unescape("a\r\nb", Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::EscapeOnlyChar(Span { start: 1, end: 2 })),
        );
        assert_eq!(
            unescape("a\r\nb", Mode::MultiLine).collect::<Result<String>>()?,
            "a\r\nb",
        );
        Ok(())
    }

    #[test]
    fn quotation_mark() -> Result<()> {
        assert_eq!(
            unescape(r#"a"b"#, Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::EscapeOnlyChar(Span { start: 1, end: 2 })),
        );
        assert_eq!(
            unescape(r#"a"b"#, Mode::MultiLine).collect::<Result<String>>()?,
            "a\"b",
        );
        Ok(())
    }

    #[test]
    fn backslash() -> Result<()> {
        assert_eq!(
            unescape(r#"a\b"#, Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::InvalidEscape(Span { start: 1, end: 3 })),
        );
        assert_eq!(
            unescape(r#"a\\b"#, Mode::MultiLine).collect::<Result<String>>()?,
            r#"a\b"#,
        );
        Ok(())
    }

    #[test]
    fn backslash_lf() -> Result<()> {
        assert_eq!(
            unescape("a\\\n    \t\n    b", Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::InvalidEscape(Span { start: 1, end: 3 })),
        );
        assert_eq!(
            unescape("a\\\n    \t\n    b", Mode::MultiLine).collect::<Result<String>>()?,
            "ab",
        );
        Ok(())
    }

    #[test]
    fn backslash_n() -> Result<()> {
        assert_eq!(
            unescape(r#"a\nb"#, Mode::SingleLine).collect::<Result<String>>()?,
            "a\nb",
        );
        assert_eq!(
            unescape(r#"a\nb"#, Mode::MultiLine).collect::<Result<String>>()?,
            "a\nb",
        );
        Ok(())
    }

    #[test]
    fn backslash_r() -> Result<()> {
        assert_eq!(
            unescape(r#"a\rb"#, Mode::SingleLine).collect::<Result<String>>()?,
            "a\rb",
        );
        assert_eq!(
            unescape(r#"a\rb"#, Mode::MultiLine).collect::<Result<String>>()?,
            "a\rb",
        );
        Ok(())
    }

    #[test]
    fn surrogate_unicode() {
        assert_eq!(
            unescape(r#"a\ud800b"#, Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::SurrogateUnicodeEscape(Span { start: 1, end: 7 })),
        );
        assert_eq!(
            unescape(r#"a\ud800b"#, Mode::MultiLine).collect::<Result<Vec<_>>>(),
            Err(Error::SurrogateUnicodeEscape(Span { start: 1, end: 7 })),
        );
    }

    #[test]
    fn out_of_range_unicode() {
        assert_eq!(
            unescape(r#"a\U00110000b"#, Mode::SingleLine).collect::<Result<Vec<_>>>(),
            Err(Error::OutOfRangeUnicodeEscape(Span { start: 1, end: 11 })),
        );
        assert_eq!(
            unescape(r#"a\U00110000b"#, Mode::MultiLine).collect::<Result<Vec<_>>>(),
            Err(Error::OutOfRangeUnicodeEscape(Span { start: 1, end: 11 })),
        );
    }
}
