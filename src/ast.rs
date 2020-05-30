use super::{
    comment::{Comment, Comments},
    key::Key,
    merge::Merge,
    value::{Item, Table, Value},
};
pub(crate) use crate::format::independent::Kind;
use std::{iter::FromIterator, mem::take, vec::Vec};

/// Lines.
pub struct Lines<'a>(Vec<Line<'a>>);

impl From<Lines<'_>> for Table {
    fn from(from: Lines) -> Self {
        let mut state = State::new();
        let comments = &mut Comments::new();
        for line in from.0 {
            match line.data {
                Some(Data::Header(key)) => {
                    comments.maybe_push(line.meta);
                    state = State::Headed {
                        comments: take(comments),
                        key,
                        inner_table: Table::new(),
                        outer_table: state.into_table(),
                    };
                }
                Some(Data::KeyValue { key, value }) => {
                    comments.maybe_push(line.meta);
                    let value = Value::wrap(key, Item::new(take(comments), value));
                    state.table_mut().merge(value);
                }
                _ => {
                    comments.maybe_push(line.meta);
                }
            }
        }
        state.into_table()
    }
}

impl<'a> FromIterator<Line<'a>> for Lines<'a> {
    fn from_iter<I: IntoIterator<Item = Line<'a>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Line.
pub struct Line<'a> {
    pub data: Option<Data<'a>>,
    pub meta: Option<Comment>,
}

/// Data.
pub enum Data<'a> {
    Header(Kind<Key<'a>>),
    KeyValue { key: Key<'a>, value: Value },
}

enum State<'a> {
    Unheaded {
        table: Table,
    },
    Headed {
        comments: Comments,
        key: Kind<Key<'a>>,
        inner_table: Table,
        outer_table: Table,
    },
}

impl<'a> State<'a> {
    fn new() -> Self {
        Self::Unheaded {
            table: Table::new(),
        }
    }

    fn into_table(self) -> Table {
        match self {
            Self::Unheaded { table } => table,
            Self::Headed {
                comments,
                key,
                inner_table,
                mut outer_table,
            } => {
                let mut item = Item::new(comments, Value::from(inner_table));
                if let Kind::ArrayOfTables(_) = key {
                    item = Item::from(Value::from(vec![item]));
                }
                let value = Value::wrap(key.into_inner(), item);
                outer_table.merge(value);
                outer_table
            }
        }
    }

    fn table_mut(&mut self) -> &mut Table {
        match self {
            Self::Unheaded { table, .. } => table,
            Self::Headed { inner_table, .. } => inner_table,
        }
    }
}
