use serde::{Serialize,Deserialize};
use lapm_core::command::entry::DaemonEntry;

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


#[cfg(test)]
mod tests {
    use keepass::Database;
    use super::PasswordEntry;

    impl PartialEq for PasswordEntry {
        fn eq(&self, other: &Self) -> bool {
            (self.title.as_deref() == other.title.as_deref()) &&
            (self.username.as_str() == other.username.as_str()) &&
            (self.password.as_str() == other.password.as_str()) &&
            (self.url.as_deref() == other.url.as_deref()) &&
            (self.notes.as_deref() == other.notes.as_deref())
        }
    }

    impl Eq for PasswordEntry {}


    #[test]
    fn set_password_entry_can_be_retrieved() {
        let mut db = Database::new();
        let mut root = db.root_mut();


    }
}
