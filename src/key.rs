use crate::{
    escape::{escape, Flags, Mode},
    quotes::{Quoted, Quotes},
};
use derive_more::{Deref, DerefMut, Display, IntoIterator};
use derive_new::new;
use itertools::Itertools;
use std::{
    borrow::Cow,
    fmt::{self, Debug, Display, Formatter},
    iter::{FromIterator, IntoIterator},
    ops::Deref,
};

/// Key.
#[derive(Clone, Debug, Default, Deref, DerefMut, Eq, Hash, IntoIterator, PartialEq, new)]
pub struct Key<'a>(#[new(default)] Vec<Segment<'a>>);

impl Display for Key<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.iter().format("."))
    }
}

impl<'a> From<Segment<'a>> for Key<'a> {
    #[inline]
    fn from(from: Segment<'a>) -> Self {
        Self(vec![from])
    }
}

impl<'a, T: Into<Segment<'a>>> FromIterator<T> for Key<'a> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

/// Segment.
#[derive(Clone, Debug, Display, Eq, Hash, PartialEq)]
pub enum Segment<'a> {
    Unquoted(Cow<'a, str>),
    Quoted(Quoted<Cow<'a, str>>),
}

impl<'a> Segment<'a> {
    pub fn new<T: Into<Cow<'a, str>>>(input: T) -> Self {
        let cow = input.into();
        let flags = Flags::parse(&cow);
        if flags.is_quoted {
            // Use only single-line.
            if flags.has_lf_or_cr || flags.has_escape || flags.has_apostrophe {
                Self::Quoted(Quoted::SingleLine(Quotes::Double(cow)))
            } else {
                Self::Quoted(Quoted::SingleLine(Quotes::Single(cow)))
            }
        } else {
            Self::Unquoted(cow)
        }
    }

    pub fn escape(&self) -> Cow<str> {
        match self {
            Self::Unquoted(cow) => Cow::Borrowed(cow),
            Self::Quoted(Quoted::SingleLine(Quotes::Single(cow))) => Cow::Borrowed(cow),
            Self::Quoted(Quoted::MultiLine(Quotes::Single(cow))) => Cow::Borrowed(cow),
            Self::Quoted(Quoted::SingleLine(Quotes::Double(cow))) => {
                escape(cow, Mode::SingleLine).collect()
            }
            Self::Quoted(Quoted::MultiLine(Quotes::Double(cow))) => {
                escape(cow, Mode::MultiLine).collect()
            }
        }
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        match self {
            Segment::Unquoted(cow) => cow,
            Segment::Quoted(Quoted::SingleLine(Quotes::Single(cow))) => cow,
            Segment::Quoted(Quoted::MultiLine(Quotes::Single(cow))) => cow,
            Segment::Quoted(Quoted::SingleLine(Quotes::Double(cow))) => cow,
            Segment::Quoted(Quoted::MultiLine(Quotes::Double(cow))) => cow,
        }
    }
}

impl Deref for Segment<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unquoted(cow) => cow,
            Self::Quoted(quoted) => quoted,
        }
    }
}

impl From<String> for Segment<'_> {
    fn from(from: String) -> Self {
        Self::new(from)
    }
}

impl<'a> From<Cow<'a, str>> for Segment<'a> {
    fn from(from: Cow<'a, str>) -> Self {
        Self::new(from)
    }
}

impl<'a> From<&'a str> for Segment<'a> {
    fn from(from: &'a str) -> Self {
        Self::new(from)
    }
}

impl From<Segment<'_>> for String {
    fn from(from: Segment) -> String {
        from.into_inner().into_owned()
    }
}

impl<'a> From<Segment<'a>> for Cow<'a, str> {
    fn from(from: Segment<'a>) -> Cow<'a, str> {
        from.into_inner()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod segment {
        use super::*;

        #[test]
        fn ht() {
            let segment = Segment::new("a\tb");
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Single(Cow::from("a\tb"))))
            );
            assert_eq!(segment.to_string(), "'a\tb'");
        }

        #[test]
        fn lf() {
            let segment = Segment::new("a\nb");
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Double(Cow::from("a\nb"))))
            );
            assert_eq!(segment.to_string(), r#""a\nb""#);
        }

        #[test]
        fn cr() {
            let segment = Segment::new("a\rb");
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Double(Cow::from("a\rb"))))
            );
            assert_eq!(segment.to_string(), r#""a\rb""#);
        }

        #[test]
        fn cr_lf() {
            let segment = Segment::new("a\r\nb");
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Double(Cow::from("a\r\nb"))))
            );
            assert_eq!(segment.to_string(), r#""a\r\nb""#);
        }

        #[test]
        fn quotation_mark() {
            let segment = Segment::new(r#"a"b"#);
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Single(Cow::from(r#"a"b"#))))
            );
            assert_eq!(segment.to_string(), r#"'a"b'"#);
        }

        #[test]
        fn tree_quotation_marks() {
            let segment = Segment::new(r#"a"""b"#);
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Single(Cow::from(r#"a"""b"#))))
            );
            assert_eq!(segment.to_string(), r#"'a"""b'"#);
        }

        #[test]
        fn apostrophe() {
            let segment = Segment::new("a'b");
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Double(Cow::from("a'b"))))
            );
            assert_eq!(segment.to_string(), r#""a'b""#);
        }

        #[test]
        fn backslash() {
            let segment = Segment::new(r#"a\b"#);
            assert_eq!(
                segment,
                Segment::Quoted(Quoted::SingleLine(Quotes::Single(Cow::from(r#"a\b"#))))
            );
            assert_eq!(segment.to_string(), r#"'a\b'"#);
        }
    }

    mod key {
        use super::*;

        #[test]
        fn test() {
            assert_eq!(Segment::new("abc"), Segment::Unquoted(Cow::from("abc")));
            assert_eq!(
                &Key::from_iter(vec![
                    Segment::Unquoted(Cow::from("a")),
                    Segment::Quoted(Quoted::SingleLine(Quotes::Single(Cow::from("b")))),
                    Segment::Quoted(Quoted::SingleLine(Quotes::Double(Cow::from("c")))),
                    Segment::Quoted(Quoted::MultiLine(Quotes::Single(Cow::from("d")))),
                    Segment::Quoted(Quoted::MultiLine(Quotes::Double(Cow::from("e")))),
                ])
                .to_string(),
                r#"a.'b'."c".'''d'''."""e""""#
            );
        }
    }

    //     #[test]
    //     fn construct() {
    //         assert_eq!(
    //             &Key::from_iter(vec![
    //                 Segment::Unquoted("a"),
    //                 Segment::Quoted(Quoted::SingleLine(Quotes::Single("b"))),
    //                 Segment::Quoted(Quoted::SingleLine(Quotes::Double("c"))),
    //                 Segment::Quoted(Quoted::MultiLine(Quotes::Single("d"))),
    //                 Segment::Quoted(Quoted::MultiLine(Quotes::Double("e"))),
    //             ])
    //             .to_string(),
    //             r#"a.'b'."c".'''d'''."""e""""#
    //         )
    //     }

    //     #[test]
    //     fn from() {
    //         //         assert_eq!(
    //         //             &Key::from_iter(vec!["a", "b.b", "c\nc", "d", "e"]).to_string(),
    //         //             r#"a."b.b"."""c
    //         // c""".d.e"#
    //         //         )
    //         println!("segment: {}", Segment::new("a'b").escape());
    //         println!("segment: {}", Segment::new("a'b\nc").escape());
    //         println!("segment: {}", Segment::new("cfg(unix)").escape());

    //         assert_eq!(
    //             r#""cfg(target_os = \"linux\")""#,
    //             Segment::Quoted(Quoted::SingleLine(Quotes::Double(
    //                 r#"cfg(target_os = "linux")"#,
    //             )))
    //             .escape()
    //             .to_string()
    //         );
    //         assert_eq!(
    //             r#""""cfg(target_os = "linux")""""#,
    //             Segment::Quoted(Quoted::MultiLine(Quotes::Double(
    //                 r#"cfg(target_os = "linux")"#,
    //             )))
    //             .escape()
    //             .to_string()
    //         );
    //     }
}
