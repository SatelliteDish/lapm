use lapm_core::command::entry::DaemonEntry;
use serde::{Serialize,Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub title: Option<String>,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

impl From<DaemonEntry> for Entry {
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

impl From<Entry> for DaemonEntry {
    fn from(entry: Entry) -> Self {
        Self {
            title: entry.title,
            username: entry.username,
            password: entry.password,
            url: entry.url,
            notes: entry.notes,
        }
    }
}

impl Entry {
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
