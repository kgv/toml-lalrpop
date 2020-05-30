use crate::{
    key::Segment,
    value::{Array, Item, Table, Value},
};
use derive_more::Deref;
use derive_new::new;
use itertools::Itertools;
use log::warn;
use pad_adapter::PadAdapter;
use std::fmt::{self, Debug, Display, Formatter, Write};

/// Inline.
#[derive(Clone, Debug, Deref, new)]
pub struct Inline<T>(T);

impl Display for Inline<Item> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&Inline::new(&self.0), f)
    }
}

impl Display for Inline<&Item> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&Inline::new(&self.value), f)
    }
}

impl Display for Inline<Value> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&Inline::new(&self.0), f)
    }
}

impl Display for Inline<&Value> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.0 {
            Value::Array(array) => Display::fmt(&Inline::new(array), f),
            Value::Table(table) => Display::fmt(&Inline::new(table), f),
            Value::Primitive(primitive) => Display::fmt(primitive, f),
        }
    }
}

impl Display for Inline<Array> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&Inline::new(&self.0), f)
    }
}

impl Display for Inline<&Array> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_char('[')?;
        if !self.is_empty() {
            if f.alternate() {
                // Alternate.
                f.write_char('\n')?;
                let mut pad_adapter = PadAdapter::new(f);
                for Item { comments, value } in self.iter() {
                    write!(pad_adapter, "{}", comments.pre())?;
                    write!(pad_adapter, "{:#},", Inline::new(value))?;
                    writeln!(pad_adapter, "{}", comments.post())?;
                }
            } else {
                // Non-alternate.
                for (index, Item { comments, value }) in self.iter().enumerate() {
                    if index != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", Inline::new(value))?;
                    if !comments.is_empty() {
                        warn!(
                            r#"comments were omitted: "{}""#,
                            comments.iter().format(r#"", ""#)
                        );
                    }
                }
            }
        }
        f.write_char(']')
    }
}

impl Display for Inline<Table> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&Inline::new(&self.0), f)
    }
}

impl Display for Inline<&Table> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_char('{')?;
        if !self.is_empty() {
            f.write_char(' ')?;
            for (index, (segment, Item { comments, value })) in self.iter().enumerate() {
                if index != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{} = ", Segment::new(segment))?;
                Display::fmt(&Inline::new(value), f)?;
                if !comments.is_empty() {
                    warn!(
                        r#"comments were omitted: "{}""#,
                        comments.iter().format(r#"", ""#)
                    );
                }
            }
            f.write_char(' ')?;
        }
        f.write_char('}')
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indexmap::indexmap;
    use std::iter::FromIterator;

    #[test]
    fn owned() {
        let _inline = Inline::new(Item::from(Value::from(Table::new())));
        let _inline = Inline::new(Value::from(Table::new()));
        let _inline = Inline::new(Array::new());
        let _inline = Inline::new(Table::new());
    }

    #[test]
    fn borrowed() {
        let _inline = Inline::new(&Item::from(Value::from(Table::new())));
        let _inline = Inline::new(&Value::from(Table::new()));
        let _inline = Inline::new(&Array::new());
        let _inline = Inline::new(&Table::new());
    }

    #[test]
    fn display() {
        let value = Value::from_iter(indexmap! {
            "c" => Value::from_iter(indexmap! {
                "cd" => Value::from_iter(indexmap! {
                    "cde" => Value::from(true),
                    "cdf" => Value::from(true),
                    "cdg" => Value::from(true),
                    "cdh" => Value::from(true),
                }),
                "ca" => Value::from(true),
                "cb" => Value::from(true),
                "cc" => Value::from(true),
            }),
            "a" => Value::from(true),
            "d" => Value::from_iter(vec![
                Value::from_iter(indexmap! {
                    "da" => Value::from_iter(vec![Value::from(true), Value::from(true)]),
                    "db" => Value::from(true),
                }),
                Value::from_iter(indexmap! {
                    "dc" => Value::from_iter(indexmap! {
                        "dca" => Value::from(true),
                        "dcb" => Value::from(true),
                    }),
                    "dd" => Value::from(true),
                }),
            ]),
            "b" => Value::from_iter(vec![Value::from(true), Value::from(true), Value::from(true), Value::from(true)]),
            "e" => Value::from(Table::new()),
            "cfg(unix)" => Value::from(true),
        });
        let inline = Inline::new(value);
        println!("inline:\n{}", inline);
        // println!("inline:\n{:#}", inline);
    }
}
