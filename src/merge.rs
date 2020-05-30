use crate::value::{Array, Table, Value};

/// Merge values.
pub(crate) trait Merge {
    fn merge(&mut self, value: Value);
}

impl Merge for Value {
    fn merge(&mut self, value: Value) {
        match self {
            Self::Table(table) => table.merge(value),
            Self::Array(array) => array.merge(value),
            _ => panic!("Can't merge a primitive value."),
        }
    }
}

impl Merge for Table {
    fn merge(&mut self, value: Value) {
        match value {
            Value::Table(other) => {
                for (segment, mut source) in other.into_iter() {
                    if let Some(target) = self.get_mut(&segment) {
                        target.value.merge(source.value);
                        target.comments.append(&mut source.comments);
                    } else {
                        self.insert(segment, source);
                    }
                }
            }
            _ => panic!("Can't merge a table value with a not-table value."),
        }
    }
}

impl Merge for Array {
    fn merge(&mut self, value: Value) {
        match value {
            Value::Array(mut other) => {
                self.append(&mut other);
            }
            _ => panic!("Can't merge an array value with a not-array value."),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indexmap::indexmap;
    use std::iter::FromIterator;

    mod primitive {
        use super::*;

        #[test]
        #[should_panic(expected = "Can't merge a primitive value.")]
        fn primitive() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from(true),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from(false),
            });
            target.merge(source);
        }

        #[test]
        #[should_panic(expected = "Can't merge a primitive value.")]
        fn array() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from(true),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from_iter(vec![Value::from(true)]),
            });
            target.merge(source);
        }

        #[test]
        #[should_panic(expected = "Can't merge a primitive value.")]
        fn table() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from(true),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from_iter(indexmap! {
                    "b" => Value::from(true),
                }),
            });
            target.merge(source);
        }
    }

    mod table {
        use super::*;

        #[test]
        #[should_panic(expected = "Can't merge a table value with a not-table value.")]
        fn primitive() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from_iter(indexmap! {
                    "b" => Value::from(true),
                }),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from(true),
            });
            target.merge(source);
        }

        #[test]
        #[should_panic(expected = "Can't merge a table value with a not-table value.")]
        fn array() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from_iter(indexmap! {
                    "b" => Value::from(true),
                }),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from_iter(vec![Value::from(true)]),
            });
            target.merge(source);
        }

        #[test]
        fn table() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from_iter(indexmap! {
                    "b" => Value::from(true),
                }),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from_iter(indexmap! {
                    "c" => Value::from(true),
                }),
            });
            target.merge(source);
            assert_eq!(
                target,
                Value::from_iter(indexmap! {
                    "a" => Value::from_iter(indexmap! {
                        "b" => Value::from(true),
                        "c" => Value::from(true),
                    }),
                }),
            );
        }
    }

    mod array {
        use super::*;

        #[test]
        #[should_panic(expected = "Can't merge an array value with a not-array value.")]
        fn primitive() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from_iter(vec![Value::from(true)]),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from(false),
            });
            target.merge(source);
        }

        #[test]
        fn array() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from_iter(vec![Value::from(true)]),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from_iter(vec![Value::from(true)]),
            });
            target.merge(source);
            assert_eq!(
                target,
                Value::from_iter(indexmap! {
                    "a" => Value::from_iter(vec![Value::from(true), Value::from(true)])
                })
            );
        }

        #[test]
        #[should_panic(expected = "Can't merge an array value with a not-array value.")]
        fn table() {
            let mut target = Value::from_iter(indexmap! {
                "a" => Value::from_iter(vec![Value::from(true)]),
            });
            let source = Value::from_iter(indexmap! {
                "a" => Value::from_iter(indexmap! {
                    "b" => Value::from(true),
                }),
            });
            target.merge(source);
        }
    }
}
