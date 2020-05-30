use super::Mode;
use std::{char::from_digit, fmt::Debug, iter::FusedIterator};

pub fn escape<'a>(input: &'a str, mode: Mode) -> impl 'a + Iterator<Item = char> {
    input.chars().flat_map(move |c| Escape::new(c, mode))
}

/// Escape.
#[derive(Clone, Debug)]
pub struct Escape {
    state: EscapeState,
}

impl Escape {
    pub fn new(c: char, mode: Mode) -> Escape {
        let state = match c {
            '\t' => EscapeState::Char(c),
            '\n' => match mode {
                Mode::SingleLine => EscapeState::Backslash('n'),
                Mode::MultiLine => EscapeState::Char(c),
            },
            '\r' => match mode {
                Mode::SingleLine => EscapeState::Backslash('r'),
                Mode::MultiLine => EscapeState::Char(c),
            },
            c if c.is_ascii_control() => EscapeState::Unicode(EscapeUnicode::new(c)),
            '"' if mode == Mode::SingleLine => EscapeState::Backslash(c),
            '\\' => EscapeState::Backslash(c),
            _ => EscapeState::Char(c),
        };
        Self { state }
    }
}

impl Iterator for Escape {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.state {
            EscapeState::Backslash(c) => {
                self.state = EscapeState::Char(c);
                Some('\\')
            }
            EscapeState::Char(c) => {
                self.state = EscapeState::Done;
                Some(c)
            }
            EscapeState::Done => None,
            EscapeState::Unicode(ref mut iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn nth(&mut self, n: usize) -> Option<char> {
        match self.state {
            EscapeState::Backslash(c) if n == 0 => {
                self.state = EscapeState::Char(c);
                Some('\\')
            }
            EscapeState::Backslash(c) if n == 1 => {
                self.state = EscapeState::Done;
                Some(c)
            }
            EscapeState::Backslash(_) => {
                self.state = EscapeState::Done;
                None
            }
            EscapeState::Char(c) => {
                self.state = EscapeState::Done;

                if n == 0 {
                    Some(c)
                } else {
                    None
                }
            }
            EscapeState::Done => None,
            EscapeState::Unicode(ref mut i) => i.nth(n),
        }
    }

    fn last(self) -> Option<char> {
        match self.state {
            EscapeState::Unicode(iter) => iter.last(),
            EscapeState::Done => None,
            EscapeState::Backslash(c) | EscapeState::Char(c) => Some(c),
        }
    }
}

impl ExactSizeIterator for Escape {
    fn len(&self) -> usize {
        match self.state {
            EscapeState::Done => 0,
            EscapeState::Char(_) => 1,
            EscapeState::Backslash(_) => 2,
            EscapeState::Unicode(ref iter) => iter.len(),
        }
    }
}

impl FusedIterator for Escape {}

/// Escape unicode.
#[derive(Clone, Debug)]
pub struct EscapeUnicode {
    c: char,
    state: EscapeUnicodeState,
    index: usize,
}

impl EscapeUnicode {
    pub fn new(c: char) -> Self {
        // the index of the most significant bit
        let bit_index = 31 - (c as u32 | 1).leading_zeros() as usize;
        // the index of the most significant hex digit
        let hex_digit_index = bit_index / 4;
        let index = if hex_digit_index < 4 { 3 } else { 7 };
        Self {
            c,
            state: EscapeUnicodeState::Backslash,
            index,
        }
    }
}

impl Iterator for EscapeUnicode {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.state {
            EscapeUnicodeState::Backslash => {
                self.state = EscapeUnicodeState::Type;
                Some('\\')
            }
            EscapeUnicodeState::Type => {
                self.state = EscapeUnicodeState::Value;
                if self.index < 4 {
                    Some('u')
                } else {
                    Some('U')
                }
            }
            EscapeUnicodeState::Value => {
                let hex_digit = ((self.c as u32) >> (self.index * 4)) & 0xf;
                let c = from_digit(hex_digit, 16).unwrap();
                if self.index == 0 {
                    self.state = EscapeUnicodeState::Done;
                } else {
                    self.index -= 1;
                }
                Some(c)
            }
            EscapeUnicodeState::Done => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<char> {
        match self.state {
            EscapeUnicodeState::Done => None,

            EscapeUnicodeState::Value
            | EscapeUnicodeState::Type
            | EscapeUnicodeState::Backslash => Some('}'),
        }
    }
}

impl ExactSizeIterator for EscapeUnicode {
    fn len(&self) -> usize {
        self.index
            + match self.state {
                EscapeUnicodeState::Done => 0,
                EscapeUnicodeState::Value => 1,
                EscapeUnicodeState::Type => 2,
                EscapeUnicodeState::Backslash => 3,
            }
    }
}

impl FusedIterator for EscapeUnicode {}

#[derive(Clone, Debug)]
enum EscapeState {
    Done,
    Char(char),
    Backslash(char),
    Unicode(EscapeUnicode),
}

#[derive(Clone, Debug)]
enum EscapeUnicodeState {
    Done,
    Value,
    Type,
    Backslash,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ht() {
        assert_eq!(escape("a\tb", Mode::SingleLine).collect::<String>(), "a\tb");
        assert_eq!(escape("a\tb", Mode::MultiLine).collect::<String>(), "a\tb");
    }

    #[test]
    fn lf() {
        assert_eq!(
            escape("a\nb", Mode::SingleLine).collect::<String>(),
            r#"a\nb"#,
        );
        assert_eq!(escape("a\nb", Mode::MultiLine).collect::<String>(), "a\nb");
    }

    #[test]
    fn cr() {
        assert_eq!(
            escape("a\rb", Mode::SingleLine).collect::<String>(),
            r#"a\rb"#,
        );
        assert_eq!(escape("a\rb", Mode::MultiLine).collect::<String>(), "a\rb");
    }

    #[test]
    fn cr_lf() {
        assert_eq!(
            escape("a\r\nb", Mode::SingleLine).collect::<String>(),
            r#"a\r\nb"#,
        );
        assert_eq!(
            escape("a\r\nb", Mode::MultiLine).collect::<String>(),
            "a\r\nb",
        );
    }

    #[test]
    fn quotation_mark() {
        assert_eq!(
            escape(r#"a"b"#, Mode::SingleLine).collect::<String>(),
            r#"a\"b"#,
        );
        assert_eq!(
            escape(r#"a"b"#, Mode::MultiLine).collect::<String>(),
            r#"a"b"#,
        );
    }

    #[test]
    fn tree_quotation_marks() {
        assert_eq!(
            escape(r#"a"""b"#, Mode::SingleLine).collect::<String>(),
            r#"a\"\"\"b"#,
        );
        assert_eq!(
            escape(r#"a"""b"#, Mode::MultiLine).collect::<String>(),
            r#"a"""b"#,
        );
    }

    #[test]
    fn backslash() {
        assert_eq!(
            escape("a\\b", Mode::SingleLine).collect::<String>(),
            r#"a\\b"#,
        );
        assert_eq!(
            escape(r#"a\b"#, Mode::MultiLine).collect::<String>(),
            r#"a\\b"#,
        );
    }
}
