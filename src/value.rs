use crate::{comment::Comments, key::Key, merge::Merge, quotes::Quoted};
use chrono::{DateTime, FixedOffset};
use derive_more::{Deref, DerefMut, Display, From, Into, IntoIterator};
use derive_new::new;
use indexmap::{indexmap, IndexMap};
use optional_index::{OptionalIndex, OptionalIndexMut};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::Debug,
    hash::Hash,
    iter::FromIterator,
    mem::discriminant,
    ops::{Index, IndexMut},
};

/// Item.
#[derive(Clone, Debug, Deref, DerefMut, PartialEq, new)]
pub struct Item {
    pub comments: Comments,
    #[deref]
    #[deref_mut]
    pub value: Value,
}

impl<I> OptionalIndex<I> for Item
where
    Value: OptionalIndex<I>,
{
    type Output = <Value as OptionalIndex<I>>::Output;

    fn optional_index(&self, index: I) -> Option<&Self::Output> {
        self.value.optional_index(index)
    }
}

impl<I> OptionalIndexMut<I> for Item
where
    Value: OptionalIndexMut<I>,
{
    fn optional_index_mut(&mut self, index: I) -> Option<&mut Self::Output> {
        self.value.optional_index_mut(index)
    }
}

impl From<Value> for Item {
    #[inline]
    fn from(from: Value) -> Self {
        Self {
            comments: Comments::new(),
            value: from,
        }
    }
}

/// Value.
#[derive(Clone, Debug, From, PartialEq)]
pub enum Value {
    Primitive(Primitive),
    Array(Array),
    Table(Table),
}

impl Value {
    /// Extracts the array value if it is an array.
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    /// Extracts the array value if it is an array.
    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    /// Extracts the boolean value if it is a boolean.
    pub fn as_boolean(&self) -> Option<&bool> {
        match self {
            Value::Primitive(Primitive::Boolean(boolean)) => Some(boolean),
            _ => None,
        }
    }

    /// Extracts the date-time value if it is a date-time.
    pub fn as_date_time(&self) -> Option<&DateTime<FixedOffset>> {
        match self {
            Value::Primitive(Primitive::DateTime(date_time)) => Some(date_time),
            _ => None,
        }
    }

    /// Extracts the float value if it is a float.
    pub fn as_float(&self) -> Option<&Float> {
        match self {
            Value::Primitive(Primitive::Float(float)) => Some(float),
            _ => None,
        }
    }

    /// Extracts the integer value if it is a integer.
    pub fn as_integer(&self) -> Option<&Integer> {
        match self {
            Value::Primitive(Primitive::Integer(integer)) => Some(integer),
            _ => None,
        }
    }

    /// Extracts the primitive value if it is a primitive.
    pub fn as_primitive(&self) -> Option<&Primitive> {
        match self {
            Value::Primitive(primitive) => Some(primitive),
            _ => None,
        }
    }

    /// Extracts the string value if it is a string.
    pub fn as_string(&self) -> Option<&Quoted<String>> {
        match self {
            Value::Primitive(Primitive::String(string)) => Some(string),
            _ => None,
        }
    }

    /// Extracts the table value if it is a table.
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }

    /// Extracts the table value if it is a table.
    pub fn as_table_mut(&mut self) -> Option<&mut Table> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }

    /// Extracts the table value if it is a table.
    pub fn into_table(self) -> Option<Table> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }

    /// Tests whether this value is a array.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Tests whether this value is a boolean.
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// Tests whether this value is a date-time.
    pub fn is_date_time(&self) -> bool {
        self.as_date_time().is_some()
    }

    /// Tests whether this value is a float.
    pub fn is_float(&self) -> bool {
        self.as_float().is_some()
    }

    /// Tests whether this value is a integer.
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    /// Tests whether this value is a primitive.
    pub fn is_primitive(&self) -> bool {
        self.as_primitive().is_some()
    }

    /// Tests whether this value is a string.
    pub fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    /// Tests whether this value is a table.
    pub fn is_table(&self) -> bool {
        self.as_table().is_some()
    }

    /// Tests whether this and another value have the same type.
    pub fn same_type(&self, other: &Value) -> bool {
        discriminant(self) == discriminant(other)
    }

    /// Returns a human-readable representation of the type of this value.
    pub fn type_str(&self) -> &'static str {
        match self {
            Value::Primitive(Primitive::String(_)) => "string",
            Value::Primitive(Primitive::Integer(_)) => "integer",
            Value::Primitive(Primitive::Float(_)) => "float",
            Value::Primitive(Primitive::Boolean(_)) => "boolean",
            Value::Primitive(Primitive::DateTime(_)) => "datetime",
            Value::Array(_) => "array",
            Value::Table(_) => "table",
        }
    }

    /// If key is empty - it is the top level table.
    pub(crate) fn wrap<'a>(mut key: Key<'a>, item: Item) -> Self {
        match key.pop() {
            Some(segment) if !key.is_empty() => {
                let value = Self::from(indexmap! { segment.into() => item });
                Self::wrap(key, Item::from(value))
            }
            Some(segment) => Self::from(indexmap! { segment.into() => item }),
            None => item.value,
        }
    }
}

impl OptionalIndex<usize> for Value {
    type Output = Item;

    fn optional_index(&self, index: usize) -> Option<&Self::Output> {
        match self {
            Self::Array(array) => array.get(index),
            _ => None,
        }
    }
}

impl<T: ?Sized> OptionalIndex<&T> for Value
where
    T: Hash + Eq,
    String: Borrow<T>,
{
    type Output = Item;

    fn optional_index(&self, index: &T) -> Option<&Self::Output> {
        match self {
            Self::Table(table) => table.get(index),
            _ => None,
        }
    }
}

impl OptionalIndexMut<usize> for Value {
    fn optional_index_mut(&mut self, index: usize) -> Option<&mut Self::Output> {
        match self {
            Self::Array(array) => array.get_mut(index),
            _ => None,
        }
    }
}

impl<T: ?Sized> OptionalIndexMut<&T> for Value
where
    T: Hash + Eq,
    String: Borrow<T>,
{
    fn optional_index_mut(&mut self, index: &T) -> Option<&mut Self::Output> {
        match self {
            Self::Table(table) => table.get_mut(index),
            _ => None,
        }
    }
}

impl From<Quoted<String>> for Value {
    #[inline]
    fn from(from: Quoted<String>) -> Self {
        Self::Primitive(Primitive::from(from))
    }
}

impl From<String> for Value {
    #[inline]
    fn from(from: String) -> Self {
        Self::from(Quoted::new(from))
    }
}

impl From<Integer> for Value {
    #[inline]
    fn from(from: Integer) -> Self {
        Self::Primitive(Primitive::from(from))
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(from: i64) -> Self {
        Self::from(Integer::from(from))
    }
}

impl From<Float> for Value {
    #[inline]
    fn from(from: Float) -> Self {
        Self::Primitive(Primitive::from(from))
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(from: f64) -> Self {
        Self::from(Float::from(from))
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(from: bool) -> Self {
        Self::Primitive(Primitive::from(from))
    }
}

impl From<DateTime<FixedOffset>> for Value {
    #[inline]
    fn from(from: DateTime<FixedOffset>) -> Self {
        Self::Primitive(Primitive::from(from))
    }
}

impl From<Vec<Item>> for Value {
    #[inline]
    fn from(from: Vec<Item>) -> Self {
        Self::Array(Array(from))
    }
}

impl From<IndexMap<String, Item>> for Value {
    #[inline]
    fn from(from: IndexMap<String, Item>) -> Self {
        Self::Table(Table(from))
    }
}

impl<T: Into<Item>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Array(Array::from_iter(iter))
    }
}

impl<K: Into<String>, V: Into<Item>> FromIterator<(K, V)> for Value {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::Table(Table::from_iter(iter))
    }
}

impl<T> Index<T> for Value
where
    Self: OptionalIndex<T>,
{
    type Output = <Self as OptionalIndex<T>>::Output;

    fn index(&self, index: T) -> &Self::Output {
        self.optional_index(index).expect("Index not found.")
    }
}

impl<T> IndexMut<T> for Value
where
    Self: OptionalIndexMut<T>,
{
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        self.optional_index_mut(index).expect("Index not found.")
    }
}

/// Primitive.
#[derive(Clone, Debug, Display, From, PartialEq)]
pub enum Primitive {
    String(Quoted<String>),
    Integer(Integer),
    Float(Float),
    Boolean(bool),
    DateTime(DateTime<FixedOffset>),
}

impl PartialOrd for Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            (Self::Integer(a), Self::Integer(b)) => a.partial_cmp(&b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(&b),
            (Self::Boolean(a), Self::Boolean(b)) => a.partial_cmp(&b),
            (Self::DateTime(a), Self::DateTime(b)) => a.partial_cmp(&b),
            _ => None,
        }
    }
}

/// Integer.
#[derive(Clone, Copy, Debug, Display)]
pub enum Integer {
    #[display(fmt = "{:#b}", _0)]
    Binary(i64),
    Decimal(i64),
    #[display(fmt = "{:#o}", _0)]
    Octal(i64),
    #[display(fmt = "{:#x}", _0)]
    Hex(i64),
}

impl From<i64> for Integer {
    #[inline]
    fn from(from: i64) -> Self {
        Self::Decimal(from)
    }
}

impl From<Integer> for i64 {
    fn from(from: Integer) -> Self {
        match from {
            Integer::Binary(v) => v,
            Integer::Decimal(v) => v,
            Integer::Octal(v) => v,
            Integer::Hex(v) => v,
        }
    }
}

impl PartialEq for Integer {
    fn eq(&self, other: &Self) -> bool {
        i64::from(*self) == i64::from(*other)
    }
}

impl PartialOrd for Integer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        i64::from(*self).partial_cmp(&i64::from(*other))
    }
}

/// Float.
#[derive(Clone, Copy, Debug, Display)]
pub enum Float {
    Decimal(f64),
    #[display(fmt = "{:#e}", _0)]
    Scientific(f64),
}

impl From<f64> for Float {
    #[inline]
    fn from(from: f64) -> Self {
        Self::Decimal(from)
    }
}

impl From<Float> for f64 {
    fn from(from: Float) -> f64 {
        match from {
            Float::Decimal(v) => v,
            Float::Scientific(v) => v,
        }
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        f64::from(*self) == f64::from(*other)
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::from(*self).partial_cmp(&f64::from(*other))
    }
}

/// Array.
#[derive(Clone, Debug, Default, Deref, DerefMut, From, Into, IntoIterator, PartialEq, new)]
pub struct Array(#[new(default)] Vec<Item>);

impl OptionalIndex<usize> for Array {
    type Output = Item;

    fn optional_index(&self, index: usize) -> Option<&Self::Output> {
        self.0.get(index)
    }
}

impl OptionalIndexMut<usize> for Array {
    fn optional_index_mut(&mut self, index: usize) -> Option<&mut Self::Output> {
        self.0.get_mut(index)
    }
}

impl<T: Into<Item>> FromIterator<T> for Array {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

/// Table.
#[derive(Clone, Debug, Default, Deref, DerefMut, From, Into, IntoIterator, PartialEq, new)]
pub struct Table(#[new(default)] IndexMap<String, Item>);

impl<T: ?Sized> OptionalIndex<&T> for Table
where
    T: Hash + Eq,
    String: Borrow<T>,
{
    type Output = Item;

    fn optional_index(&self, index: &T) -> Option<&Self::Output> {
        self.0.get(index)
    }
}

impl<T: ?Sized> OptionalIndexMut<&T> for Table
where
    T: Hash + Eq,
    String: Borrow<T>,
{
    fn optional_index_mut(&mut self, index: &T) -> Option<&mut Self::Output> {
        self.0.get_mut(index)
    }
}

impl<K: Into<String>, V: Into<Item>> FromIterator<(K, V)> for Table {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl<'a> FromIterator<(Key<'a>, Value)> for Table {
    fn from_iter<I: IntoIterator<Item = (Key<'a>, Value)>>(iter: I) -> Self {
        let mut table = Self::new();
        for (key, value) in iter {
            table.merge(Value::wrap(key, Item::from(value)));
        }
        table
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::{Array, Primitive, Table, Value};

    impl From<Value> for toml::Value {
        fn from(from: Value) -> Self {
            match from {
                Value::Primitive(Primitive::String(string)) => Self::String(string.to_string()),
                Value::Primitive(Primitive::Integer(integer)) => Self::Integer(integer.into()),
                Value::Primitive(Primitive::Float(float)) => Self::Float(float.into()),
                Value::Primitive(Primitive::Boolean(boolean)) => Self::Boolean(boolean),
                // TODO:
                // Value::Primitive(Primitive::DateTime(date_time)) => Self::Datetime(date_time),
                Value::Array(array) => Self::Array(array.into()),
                Value::Table(table) => Self::Table(table.into()),
                _ => unimplemented!(),
            }
        }
    }

    impl From<Array> for toml::value::Array {
        fn from(from: Array) -> Self {
            from.0.into_iter().map(|v| v.value.into()).collect()
        }
    }

    impl From<Table> for toml::value::Table {
        fn from(from: Table) -> Self {
            from.0
                .into_iter()
                .map(|(k, v)| (k, v.value.into()))
                .collect()
        }
    }

    // impl From<toml::Value> for Value {
    //     fn from(from: toml::Value) -> Value {
    //         match from {
    //             toml::Value::String(string) => {
    //                 Self::Primitive(Primitive::String(string.to_string()))
    //             }
    //             toml::Value::Integer(integer) => {
    //                 Self::Primitive(Primitive::Integer(integer.into()))
    //             }
    //             toml::Value::Float(float) => Value::Primitive(Primitive::Float(float.into())),
    //             toml::Value::Boolean(boolean) => Value::Primitive(Primitive::Boolean(boolean)),
    //             // toml::Value::Datetime(date_time) => {
    //             //     Self::Primitive(Primitive::DateTime(date_time))
    //             // }
    //             toml::Value::Array(array) => Self::Array(array.into()),
    //             toml::Value::Table(table) => Self::Table(table.into()),
    //             _ => unimplemented!(),
    //         }
    //     }
    // }

    // impl From<toml::value::Array> for Array {
    //     fn from(from: toml::value::Array) -> Array {
    //         from.into_iter().map(|v| v.into()).collect()
    //     }
    // }

    // impl From<toml::value::Table> for Table {
    //     fn from(from: toml::value::Table) -> Table {
    //         from.into_iter()
    //             .map(|(k, v)| (k.into(), v.into()))
    //             .collect()
    //     }
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let a = Primitive::Boolean(true);
        let b = Primitive::Integer(Integer::Decimal(9));
        assert_eq!(a.partial_cmp(&b), None);
        let a = Primitive::Integer(Integer::Decimal(9));
        let b = Primitive::Integer(Integer::Hex(9));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
        let a = Primitive::Integer(Integer::Decimal(9));
        let b = Primitive::Integer(Integer::Hex(8));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
        let a = Primitive::Integer(Integer::Decimal(9));
        let b = Primitive::Integer(Integer::Hex(10));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));

        println!("0: {:?}", a.partial_cmp(&b));
    }

    #[test]
    fn array() {
        let value = Value::from_iter(vec![Value::from(true), Value::from(true)]);
        println!("0: {:?}", &value[0]);
        println!("1: {:?}", &value[1]);
    }

    #[test]
    fn table() {
        let value = Value::from_iter(indexmap! {
            "a" => Value::from(true),
            "b" => Value::from(true),
        });
        println!("a: {:?}", value["a"]);
        println!("b: {:?}", value["b"]);
    }
}
