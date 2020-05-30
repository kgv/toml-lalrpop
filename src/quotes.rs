use crate::escape::{escape, Flags, Mode};
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
};

/// Quoted.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Quoted<T> {
    SingleLine(Quotes<T>),
    MultiLine(Quotes<T>),
}

impl<T> Quoted<T> {
    pub fn new(input: T) -> Self
    where
        T: AsRef<str>,
    {
        let flags = Flags::parse(input.as_ref());
        if flags.has_escape || flags.has_apostrophe {
            if flags.has_lf_or_cr {
                Self::MultiLine(Quotes::Double(input))
            } else {
                Self::SingleLine(Quotes::Double(input))
            }
        } else {
            if flags.has_lf_or_cr {
                Self::MultiLine(Quotes::Single(input))
            } else {
                Self::SingleLine(Quotes::Single(input))
            }
        }
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> Quoted<U> {
        match self {
            Self::SingleLine(quotes) => Quoted::SingleLine(quotes.map(f)),
            Self::MultiLine(quotes) => Quoted::MultiLine(quotes.map(f)),
        }
    }
}

impl<T> Deref for Quoted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::SingleLine(quotes) => quotes,
            Self::MultiLine(quotes) => quotes,
        }
    }
}

impl<T: AsRef<str>> Display for Quoted<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::SingleLine(Quotes::Single(t)) => {
                let str = t.as_ref();
                Display::fmt(&Quotes::Single(str), f)
            }
            Self::MultiLine(Quotes::Single(t)) => {
                let str = t.as_ref();
                Display::fmt(&Quotes::Single(Quotes::Single(Quotes::Single(str))), f)
            }
            Self::SingleLine(Quotes::Double(t)) => {
                let string: String = escape(t.as_ref(), Mode::SingleLine).collect();
                Display::fmt(&Quotes::Double(string), f)
            }
            Self::MultiLine(Quotes::Double(t)) => {
                let string: String = escape(t.as_ref(), Mode::MultiLine).collect();
                Display::fmt(&Quotes::Double(Quotes::Double(Quotes::Double(string))), f)
            }
        }
    }
}

/// Quotes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Quotes<T> {
    Single(T),
    Double(T),
}

impl<T> Quotes<T> {
    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> Quotes<U> {
        match self {
            Self::Single(t) => Quotes::Single(f(t)),
            Self::Double(t) => Quotes::Double(f(t)),
        }
    }
}

impl<T> Deref for Quotes<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Single(t) => t,
            Self::Double(t) => t,
        }
    }
}

impl<T: Display> Display for Quotes<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Quotes::Single(t) => write!(f, "'{}'", t),
            Quotes::Double(t) => write!(f, r#""{}""#, t),
        }
    }
}
