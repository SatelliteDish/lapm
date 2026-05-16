use keepass::db::{GroupMut, fields};
use lapm_core::command::entry::DaemonEntry;
use serde::{Serialize,Deserialize};
use thiserror::Error;

use crate::layer::LayerError;



#[derive(Debug,Clone,Error)]
pub enum EntryError {
    #[error("Can't serialize {value} as {value_t}")]
    SerializationError {
        value: String,
        value_t: &'static str,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub title: Option<String>,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

impl From<DaemonEntry> for PasswordEntry {
    fn from(entry: DaemonEntry) -> Self {
        Self {
            title: entry.title,
            username: entry.username,
            password: entry.password,
            url: entry.url,
            notes: entry.notes,
        }
    }
}

impl From<PasswordEntry> for DaemonEntry {
    fn from(entry: PasswordEntry) -> Self {
        Self {
            title: entry.title,
            username: entry.username,
            password: entry.password,
            url: entry.url,
            notes: entry.notes,
        }
    }
}

impl PasswordEntry {
    pub fn new(username: String, password: String) -> Self {
        Self {
            title: None,
            username,
            password,
            url: None,
            notes: None,
        }
    }
}


pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

impl ConfigEntry {
    pub fn get(group: &mut GroupMut, key: String) -> Option<Self> {
        let ent = group.entry_by_name_mut(&key)?;
        let value = ent.get("value")?;

        Some(Self {
            key,
            value: value.to_string(),
        })
    }

    pub fn set(self, group: &mut GroupMut) {
        match group.entry_by_name_mut(&self.key) {
            Some(ent) => ent,
            None => {
                let mut ent = group.add_entry();
                ent.set_unprotected(fields::TITLE, &self.key);
                ent
            },
        }.set_unprotected("value", &self.value);
    }

    /*
    *  Conversion functions, to convert self.value to various types
    *  or to construct Self from various values types
    * */

    pub fn from_bool(key: String, value: bool) -> Self {
        Self {
            key,
            value: value.to_string(),
        }
    }

    pub fn to_bool(self) -> Result<bool, EntryError> {
        let str = self.value.as_str();
        if str == "true" {
            Ok(true)
        } else if str == "false" {
            Ok(false)
        } else {
            Err(EntryError::SerializationError { value: self.value, value_t: "bool" })
        }
    }
}
