// FIXME: [infer type for a closure argument](https://github.com/rust-lang/rust/issues/41078)

use super::inline::Inline;
use crate::{
    comment::Comments,
    key::{Key, Segment},
    value::{Array, Item, Table, Value},
};
use derive_new::new;
use itertools::{Either, Itertools};
use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display, Formatter},
    iter::FromIterator,
    vec,
};

/// Independent.
///
/// Only the table value can be independent.
#[derive(Clone, Copy, Debug)]
pub struct Independent<'a, T, F, G = F> {
    branch: Option<&'a Branch<'a, G>>,
    comments: Option<&'a Comments>,
    table: T,
    is_inline: F,
}

impl<T, F: Fn(&[&str]) -> bool> Independent<'_, T, F> {
    pub fn new(table: T, is_inline: F) -> Self {
        Self {
            branch: None,
            comments: None,
            table,
            is_inline,
        }
    }
}

impl<T, F, G> Display for Independent<'_, T, F, G>
where
    T: Borrow<Table>,
    F: Borrow<G>,
    G: Fn(&[&str]) -> bool,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (leafs, branches) = self
            .table
            .borrow()
            .partition(self.branch, self.is_inline.borrow());
        if let Some(branch) = self.branch {
            if !leafs.is_empty() || branches.is_empty() {
                writeln!(f)?;
                if let Some(comments) = self.comments {
                    write!(f, "{}", comments.pre())?;
                }
                match branch.header() {
                    Kind::ArrayOfTables(key) => write!(f, "[[{}]]", Key::from_iter(key))?,
                    Kind::Table(key) => write!(f, "[{}]", Key::from_iter(key))?,
                }
                if let Some(comments) = self.comments {
                    write!(f, "{}", comments.post())?;
                }
                writeln!(f)?;
            }
        }
        for leaf in leafs {
            Display::fmt(&leaf, f)?;
        }
        for branch in branches {
            Display::fmt(&branch, f)?;
        }
        Ok(())
    }
}

/// Independent kind.
#[derive(Clone, Copy, Debug)]
pub enum Kind<T, U = T> {
    ArrayOfTables(T),
    Table(U),
}

impl<T> Kind<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::ArrayOfTables(t) => t,
            Self::Table(t) => t,
        }
    }
}

/// Array of tables.
trait ArrayOfTables {
    fn as_array_of_tables(&self) -> Option<Vec<(&Comments, &Table)>>;
    fn is_array_of_tables(&self) -> bool;
}

impl ArrayOfTables for Array {
    /// Extracts the array of tables if it is an array of tables.
    fn as_array_of_tables(&self) -> Option<Vec<(&Comments, &Table)>> {
        self.iter()
            .map(|Item { comments, value }| {
                let table = value.as_table()?;
                Some((comments, table))
            })
            .collect()
    }

    /// Tests whether this array is a array of tables.
    fn is_array_of_tables(&self) -> bool {
        !self.is_empty() && self.iter().all(|Item { value, .. }| value.is_table())
    }
}

/// Partition.
trait Partition<'a> {
    fn partition<F: Fn(&[&str]) -> bool>(
        &'a self,
        branch: Option<&'a Branch<F>>,
        is_inline: &'a F,
    ) -> (Vec<Leaf>, Vec<Branch<F>>);
}

impl<'a> Partition<'a> for Table {
    fn partition<F: Fn(&[&str]) -> bool>(
        &'a self,
        branch: Option<&'a Branch<F>>,
        is_inline: &'a F,
    ) -> (Vec<Leaf>, Vec<Branch<F>>) {
        self.iter()
            .partition_map(move |(segment, Item { comments, value })| {
                let key = branch
                    .map(|branch| {
                        let mut key = branch.key();
                        key.push(segment);
                        key
                    })
                    .unwrap_or(vec![segment]);
                match value {
                    Value::Array(array) if array.is_array_of_tables() && !is_inline(&key) => {
                        let array_of_tables = array.as_array_of_tables().unwrap();
                        Either::Right(Branch::new(
                            branch,
                            comments,
                            segment,
                            Kind::ArrayOfTables(array_of_tables),
                            is_inline,
                        ))
                    }
                    Value::Table(table) if !is_inline(&key) => Either::Right(Branch::new(
                        branch,
                        comments,
                        segment,
                        Kind::Table(table),
                        is_inline,
                    )),
                    _ => Either::Left(Leaf::new(comments, segment, value)),
                }
            })
    }
}

/// Leaf.
#[derive(Clone, Copy, Debug, new)]
struct Leaf<'a> {
    comments: &'a Comments,
    segment: &'a str,
    value: &'a Value,
}

impl Display for Leaf<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.comments.pre())?;
        write!(f, "{} = ", Segment::new(self.segment))?;
        Display::fmt(&Inline::new(self.value), f)?;
        writeln!(f, "{}", self.comments.post())
    }
}

/// Branch.
#[derive(Clone, Debug, new)]
struct Branch<'a, F> {
    parent: Option<&'a Branch<'a, F>>,
    comments: &'a Comments,
    segment: &'a str,
    value: Kind<Vec<(&'a Comments, &'a Table)>, &'a Table>,
    is_inline: &'a F,
}

impl<F> Branch<'_, F> {
    fn header(&self) -> Kind<Vec<&str>> {
        match self.value {
            Kind::ArrayOfTables(_) => Kind::ArrayOfTables(self.key()),
            Kind::Table(_) => Kind::Table(self.key()),
        }
    }

    fn key(&self) -> Vec<&str> {
        match self.parent {
            Some(parent) => {
                let mut key = parent.key();
                key.push(self.segment);
                key
            }
            None => vec![self.segment],
        }
    }
}

impl<F: Fn(&[&str]) -> bool> Display for Branch<'_, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.value {
            Kind::ArrayOfTables(array_of_tables) => {
                for (comments, table) in array_of_tables.iter() {
                    let independent = Independent {
                        branch: Some(self),
                        comments: Some(comments),
                        table: *table,
                        is_inline: self.is_inline,
                    };
                    Display::fmt(&independent, f)?;
                }
            }
            Kind::Table(table) => {
                let independent = Independent {
                    branch: Some(self),
                    comments: Some(self.comments),
                    table: *table,
                    is_inline: self.is_inline,
                };
                Display::fmt(&independent, f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indexmap::indexmap;
    use std::iter::FromIterator;

    #[test]
    fn owned() {
        let is_inline = |_key: &[&str]| true;
        let _independent = Independent::new(Table::new(), is_inline);
        let _independent = Independent::new(Table::new(), &is_inline);
    }

    #[test]
    fn owned_inline() {
        let _independent = Independent::new(Table::new(), |_key| true);
    }

    #[test]
    fn borrowed() {
        let is_inline = |_key: &[&str]| true;
        let _independent = Independent::new(&Table::new(), is_inline);
        let _independent = Independent::new(&Table::new(), &is_inline);
    }

    #[test]
    fn borrowed_inline() {
        let _independent = Independent::new(&Table::new(), |_key| true);
    }

    #[test]
    fn display() {
        let table = Table::from_iter(indexmap! {
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
                    "dc" => Value::from_iter(indexmap! {
                        "dca" => Value::from(true),
                        "dcb" => Value::from(true),
                    }),
                    "da" => Value::from_iter(vec![Value::from(true), Value::from(true)]),
                    "db" => Value::from(true),
                }),
                Value::from_iter(indexmap! {
                    "dd" => Value::from_iter(indexmap! {
                        "dca" => Value::from(true),
                        "dcb" => Value::from(true),
                    }),
                    "de" => Value::from(true),
                }),
            ]),
            "b" => Value::from_iter(vec![Value::from(true), Value::from(true), Value::from(true), Value::from(true)]),
            "e" => Value::from(Table::new()),
        });
        let is_inline = |key: &[&str]| match key {
            ["c"] => false,
            ["c", ..] => true,
            ["d"] => false,
            ["d", ..] => true,
            _ => false,
        };
        let independent = Independent::new(table, is_inline);
        println!("independent:\n{}", independent);
        // println!("independent:\n{:#}", independent);
    }

    #[test]
    fn test() {
        let is_inline = |_key: &[&str]| true;
        let independent = Independent::new(Table::new(), is_inline);
        println!("independent: {}", independent);

        let independent = Independent::new(Table::new(), |_key| true);
        println!("independent: {}", independent);
    }
}
