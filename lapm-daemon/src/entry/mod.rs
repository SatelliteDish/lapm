use keepass::db::{EntryMut, GroupRef, GroupMut};
use thiserror::Error;
use derive_more::Display;

pub mod password;
pub use password::PasswordEntry;

pub mod config;

#[derive(Debug,Clone,Display)]
#[display("{name} {reason}")]
pub struct FieldError {
    name: &'static str,
    reason: String,
}

#[derive(Debug,Clone,Error)]
pub enum EntryError {
    #[error("Can't serialize {value} as {value_t}")]
    SerializationError {
        value: String,
        value_t: &'static str,
    },
    #[error(
        "Cannot deserialize fields: {}",
        fields.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )]
    InvalidFields {
        fields: Vec<FieldError>,
    }
}


pub trait Entry<'a>: Sized {
    type Key;

    fn get(group: &'a mut GroupMut<'a>, key: Self::Key) -> Option<Self>;
    fn delete(self);

    fn inner_mut(&'a mut self) -> &'a mut EntryMut<'a>;

    fn write_opt_unprotected(&'a mut self, key: &str, value: Option<String>) {
        if let Some(val) = value {
            self.inner_mut().set_unprotected(key, &val);
        } else {
            self.inner_mut().fields.remove(key);
        }
    }
}


fn set_opt_unprotected(entry: &mut EntryMut<'_>, key: &str, value: Option<String>) {
    if let Some(val) = value {
        entry.set_unprotected(key, val);
    } else {
        entry.fields.remove(key);
    }
}

pub trait Insert {
    type Response;

    fn insert(self, group: &mut GroupMut<'_>) -> Self::Response;
}

pub enum DeleteResponse {
    Success,
    NotFound,
    TooMany(usize),
}

pub trait Query {
    type Response;

    fn query(&self, group: &GroupRef<'_>) -> Vec<Self::Response>;
    fn delete(&self, group: & mut GroupMut<'_>) -> DeleteResponse;
}

pub trait Update {
    fn update(self, group: &mut GroupMut) -> Option<()>;
}
