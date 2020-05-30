use derive_more::{Deref, DerefMut, From, Into};
use derive_new::new;
use std::{
    borrow::Borrow,
    fmt::{self, Display, Formatter},
};

/// Comment.
pub type Comment = Kind<String>;

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Pre(comment) => write!(f, "#{}", comment),
            Self::Post(comment) => write!(f, "#{}", comment),
        }
    }
}

/// Comments.
#[derive(Clone, Debug, Default, Deref, DerefMut, From, Into, PartialEq, new)]
pub struct Comments(#[new(default)] Vec<Comment>);

impl Comments {
    pub(crate) fn maybe_push(&mut self, comment: Option<Comment>) {
        if let Some(comment) = comment {
            self.0.push(comment);
        }
    }

    pub(crate) fn pre(&self) -> Kind<Vec<&str>> {
        Kind::Pre(
            self.iter()
                .filter_map(|comment| match comment {
                    Kind::Pre(comment) => Some(&**comment),
                    _ => None,
                })
                .collect(),
        )
    }

    pub(crate) fn post(&self) -> Kind<Vec<&str>> {
        Kind::Post(
            self.iter()
                .filter_map(|comment| match comment {
                    Kind::Post(comment) => Some(&**comment),
                    _ => None,
                })
                .collect(),
        )
    }
}

/// Kind.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind<T> {
    Pre(T),
    Post(T),
}

impl<T> Kind<T> {
    pub fn is_pre(&self) -> bool {
        match self {
            Self::Pre(_) => true,
            Self::Post(_) => false,
        }
    }

    pub fn is_post(&self) -> bool {
        match self {
            Self::Pre(_) => false,
            Self::Post(_) => true,
        }
    }
}

impl<T> Display for Kind<Vec<T>>
where
    T: Borrow<str> + Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Kind::Pre(comments) => {
                for comment in comments {
                    writeln!(f, "#{}", comment)?;
                }
            }
            Kind::Post(comments) => {
                if !comments.is_empty() {
                    write!(f, " #{}", comments.join(" "))?;
                }
            }
        }
        Ok(())
    }
}
