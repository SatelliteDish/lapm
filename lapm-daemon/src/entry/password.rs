use lapm_core::command::entry::DaemonEntry;
use keepass::db::{
    EntryId, EntryMut as KeePassEntryMut, EntryRef as KeePassEntryRef, GroupMut, GroupRef, fields
};
use uuid::Uuid;

use crate::entry::{FieldError, Query, Update, set_opt_unprotected};

use super::{
    EntryError,
    Insert,
    DeleteResponse,
};

#[derive(Debug,Clone)]
pub struct PasswordEntry {
    pub title: Option<String>,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    id: EntryId,
}

impl PasswordEntry {
    pub fn id(&self) -> &EntryId {
        &self.id
    }
}

impl TryFrom<KeePassEntryMut<'_>> for PasswordEntry {
    type Error = EntryError;

    fn try_from(value: KeePassEntryMut<'_>) -> Result<Self, Self::Error> {
        let username = value.get(fields::USERNAME);
        let password = value.get(fields::PASSWORD);

        if let (Some(username), Some(password)) = (username, password) {
            Ok(Self {
                title: value.get(fields::TITLE).map(|str| str.to_string()),
                username: username.to_string(),
                password: password.to_string(),
                url: value.get(fields::URL).map(|str| str.to_string()),
                notes: value.get(fields::NOTES).map(|str| str.to_string()),
                id: value.id(),
            })
        } else {
            let mut errs: Vec<FieldError> = vec![];

            if username.is_none() {
                errs.push(FieldError {
                    name: "username",
                    reason: "is required".to_string(),
                });
            }
            if password.is_none() {
                errs.push(FieldError {
                    name: "password",
                    reason: "is required".to_string(),
                });
            }

            Err(EntryError::InvalidFields { fields: errs })
        }

    }
}

impl TryFrom<KeePassEntryRef<'_>> for PasswordEntry {
    type Error = EntryError;

    fn try_from(value: KeePassEntryRef<'_>) -> Result<Self, Self::Error> {
        let username = value.get(fields::USERNAME);
        let password = value.get(fields::PASSWORD);

        if let (Some(username), Some(password)) = (username, password) {
            Ok(Self {
                title: value.get(fields::TITLE).map(|str| str.to_string()),
                username: username.to_string(),
                password: password.to_string(),
                url: value.get(fields::URL).map(|str| str.to_string()),
                notes: value.get(fields::NOTES).map(|str| str.to_string()),
                id: value.id(),
            })
        } else {
            let mut errs: Vec<FieldError> = vec![];

            if username.is_none() {
                errs.push(FieldError {
                    name: "username",
                    reason: "is required".to_string(),
                });
            }
            if password.is_none() {
                errs.push(FieldError {
                    name: "password",
                    reason: "is required".to_string(),
                });
            }

            Err(EntryError::InvalidFields { fields: errs })
        }

    }
}

#[derive(Debug,Clone)]
pub struct QueryPasswordEntry<'q> {
    pub id: Option<&'q Uuid>,
    pub username: Option<&'q str>,
    pub title: Option<Option<&'q str>>,
    pub url: Option<Option<&'q str>>,
    pub notes: Option<Option<&'q str>>,
}

impl PartialEq<KeePassEntryRef<'_>> for QueryPasswordEntry<'_> {
    fn eq(&self, other: &KeePassEntryRef) -> bool {
        if let Some(q_id) = self.id {
            if q_id != &other.id().uuid() {
                return false;
            }
        }

        if let Some(q_user) = self.username {
            if Some(q_user) != other.get(fields::USERNAME) {
                return false;
            }
        }

        if let Some(q_title) = self.title {
            if q_title != other.get(fields::TITLE) {
                return false;
            }
        }

        if let Some(q_url) = self.url {
            if q_url != other.get(fields::URL) {
                return false;
            }
        }

        if let Some(q_notes) = self.notes {
            if q_notes != other.get(fields::NOTES) {
                return false;
            }
        }

        true
    }
}

impl Default for QueryPasswordEntry<'_> {
    fn default() -> Self {
        Self { id: None, username: None, title: None, url: None, notes: None }
    }
}

impl Query for QueryPasswordEntry<'_> {
    type Response = PasswordEntry;

    fn query(&self, group: &GroupRef) -> Vec<Self::Response> {
        let ids = group.entry_ids();
        let mut res: Vec<PasswordEntry> = vec![];

        for id in ids {
            // ID was just queried so we know it's there
            let ent = group.entry(id).unwrap();
            if self == &ent {
                if let Ok(pwd) = PasswordEntry::try_from(ent) {
                    res.push(pwd);
                }
            }
        }

        res
    }

    fn delete(&self, group: &mut GroupMut<'_>) -> DeleteResponse {
        let found = self.query(&group.as_ref());
        match found.len() {
            0 => DeleteResponse::NotFound,
            1 => {
                group.entry_mut(*found[0].id())
                    .unwrap().remove(); // Can safely unwrap
                DeleteResponse::Success
            },
            _ => DeleteResponse::TooMany(found.len())
        }
    }
}


#[derive(Debug,Clone)]
pub struct InsertPasswordEntry {
    pub title: Option<String>,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

impl From<DaemonEntry> for InsertPasswordEntry {
    fn from(value: DaemonEntry) -> Self {
        Self {
            title: value.title,
            username: value.username,
            password: value.password,
            url: value.url,
            notes: value.notes,
        }
    }
}

impl Insert for InsertPasswordEntry {
    type Response = PasswordEntry;

    fn insert(self, group: &mut GroupMut<'_>) -> PasswordEntry {
        let Self { title, username, password, url, notes } = self;
        let mut ent = group.add_entry();

        ent.set_unprotected(fields::USERNAME, &username);
        ent.set_protected(fields::PASSWORD, &password);

        if let Some(title) = &title {
            ent.set_unprotected(fields::TITLE, title);
        }

        if let Some(url) = &url {
            ent.set_unprotected(fields::URL, url);
        }

        if let Some(notes) = &notes {
            ent.set_unprotected(fields::NOTES, notes);
        }

        PasswordEntry {
            title,
            username,
            password,
            url,
            notes,
            id: ent.id(),
        }
    }
}


#[derive(Debug,Clone)]
pub struct UpdatePasswordEntry<'e> {
    pub id: &'e Uuid,
    pub username: Option<String>,
    pub password: Option<String>,
    pub title: Option<Option<String>>,
    pub url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
}

impl<'e> Update for UpdatePasswordEntry<'e> {
    fn update(self, group: &mut GroupMut) -> Option<()> {
        let Self { id, username, password, title, url, notes } = self;
        let ent_id = group.entry_ids().find(|i| id == &i.uuid())?;
        let mut ent = group.entry_mut(ent_id)?;


        if let Some(user) = username {
            ent.set_unprotected(fields::USERNAME, user);
        }
        if let Some(pass) = password {
            ent.set_protected(fields::PASSWORD, pass);
        }

        if let Some(title_opt) = title {
            set_opt_unprotected(&mut ent, fields::TITLE, title_opt);
        }
        if let Some(url_opt) = url {
            set_opt_unprotected(&mut ent, fields::URL, url_opt);
        }
        if let Some(notes_opt) = notes {
            set_opt_unprotected(&mut ent, fields::NOTES, notes_opt);
        }

        Some(())
    }
}



#[cfg(test)]
mod tests {
    use keepass::Database;

    use crate::test_helpers::{
        random_string,
        random_opt,
    };
    use super::*;

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

    fn get_test_insert(count: usize) -> Vec<InsertPasswordEntry> {
        (0..count).map(|_| InsertPasswordEntry {
            username: random_string(10),
            password: random_string(10),
            title: random_opt(random_string(15)),
            url: random_opt(random_string(15)),
            notes: random_opt(random_string(15)),
        }).collect::<Vec<_>>()
    }

    fn insert_test_entries(group: &mut GroupMut<'_>, count: usize) -> Vec<PasswordEntry> {
        get_test_insert(count)
            .into_iter()
            .map(|ins| ins.insert(group))
            .collect::<Vec<_>>()
    }

    #[test]
    fn set_entry_can_be_retrieved() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let entries = insert_test_entries(&mut root, 25);

        for entry in entries {
            let res = QueryPasswordEntry {
                id: Some(&entry.id().uuid()),
                username: None,
                title: None,
                url: None,
                notes: None,
            }.query(&root.as_ref());
            assert_eq!(res.len(), 1);
            assert_eq!(res[0], entry);
        }
    }

    #[test]
    fn update_entry_changes_values() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let entries: Vec<PasswordEntry> = insert_test_entries(&mut root, 25);
        for ent in entries {
            let new = get_test_insert(1).pop().unwrap();

            let update = UpdatePasswordEntry {
                id: &ent.id().uuid(),
                username: random_opt(new.username),
                password: random_opt(new.password),
                title: random_opt(new.title),
                url: random_opt(new.url),
                notes: random_opt(new.notes),
            };
            update.clone().update(&mut root);

            let result = QueryPasswordEntry {
                id: Some(&ent.id().uuid()),
                ..Default::default()
            }.query(&root.as_ref())
                .pop().unwrap();

            assert_eq!(result.username, if let Some(username) = update.username { username } else { ent.username } );
            assert_eq!(result.password, if let Some(password) = update.password { password } else { ent.password } );
            assert_eq!(result.title, if let Some(title) = update.title { title } else { ent.title } );
            assert_eq!(result.url, if let Some(url) = update.url { url } else { ent.url } );
            assert_eq!(result.notes, if let Some(notes) = update.notes { notes } else { ent.notes } );
        }
    }

    #[test]
    fn deleted_entries_not_found() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let entries: Vec<PasswordEntry> = insert_test_entries(&mut root, 25);

        for entry in entries {
            QueryPasswordEntry {
                id: Some(&entry.id().uuid()),
                ..Default::default()
            }.delete(&mut root);

            let query_response = QueryPasswordEntry {
                id: Some(&entry.id().uuid()),
                ..Default::default()
            }.query(&root.as_ref());
            assert!(query_response.is_empty());
        }
    }

    #[test]
    fn query_field_combinations_return_correct_matches() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        // Fixed population with deliberate overlap across every field
        let inserts = vec![
            InsertPasswordEntry { username: "alice".into(), password: "p1".into(), title: Some("Bank".into()),    url: Some("https://bank.com".into()),  notes: Some("note1".into()) },
            InsertPasswordEntry { username: "alice".into(), password: "p2".into(), title: Some("Bank".into()),    url: Some("https://other.com".into()), notes: None },
            InsertPasswordEntry { username: "alice".into(), password: "p3".into(), title: None,                   url: Some("https://bank.com".into()),  notes: Some("note1".into()) },
            InsertPasswordEntry { username: "bob".into(),   password: "p4".into(), title: Some("Bank".into()),    url: Some("https://bank.com".into()),  notes: None },
            InsertPasswordEntry { username: "bob".into(),   password: "p5".into(), title: None,                   url: None,                             notes: Some("note1".into()) },
            InsertPasswordEntry { username: "carol".into(), password: "p6".into(), title: Some("Work".into()),    url: None,                             notes: None },
            InsertPasswordEntry { username: "carol".into(), password: "p7".into(), title: None,                   url: None,                             notes: None },
            InsertPasswordEntry { username: "dave".into(),  password: "p8".into(), title: Some("Bank".into()),    url: Some("https://other.com".into()), notes: Some("note1".into()) },
            InsertPasswordEntry { username: "dave".into(),  password: "p9".into(), title: Some("Work".into()),    url: Some("https://bank.com".into()),  notes: None },
            InsertPasswordEntry { username: "eve".into(),   password: "p10".into(), title: None,                  url: None,                             notes: None },
        ];

        let entries: Vec<PasswordEntry> = inserts.iter().cloned()
            .map(|i| i.insert(&mut root))
            .collect();

        // Each tuple: (query, expected ids by index into `entries`)
        // Indices are verified by checking the returned set matches exactly
        let cases: Vec<(QueryPasswordEntry, Vec<usize>)> = vec![
            // Single-field: username
            (QueryPasswordEntry { username: Some("alice"), ..Default::default() },   vec![0,1,2]),
            (QueryPasswordEntry { username: Some("bob"),   ..Default::default() },   vec![3,4]),
            (QueryPasswordEntry { username: Some("carol"), ..Default::default() },   vec![5,6]),
            (QueryPasswordEntry { username: Some("dave"),  ..Default::default() },   vec![7,8]),
            (QueryPasswordEntry { username: Some("eve"),   ..Default::default() },   vec![9]),
            (QueryPasswordEntry { username: Some("ghost"), ..Default::default() },   vec![]),

            // Single-field: title (Some(Some(...)) vs Some(None))
            (QueryPasswordEntry { title: Some(Some("Bank")), ..Default::default() }, vec![0,1,3,7]),
            (QueryPasswordEntry { title: Some(Some("Work")), ..Default::default() }, vec![5,8]),
            (QueryPasswordEntry { title: Some(None),         ..Default::default() }, vec![2,4,6,9]),

            // Single-field: url
            (QueryPasswordEntry { url: Some(Some("https://bank.com")),  ..Default::default() }, vec![0,2,3,8]),
            (QueryPasswordEntry { url: Some(Some("https://other.com")), ..Default::default() }, vec![1,7]),
            (QueryPasswordEntry { url: Some(None),                       ..Default::default() }, vec![4,5,6,9]),

            // Single-field: notes
            (QueryPasswordEntry { notes: Some(Some("note1")), ..Default::default() }, vec![0,2,4,7]),
            (QueryPasswordEntry { notes: Some(None),          ..Default::default() }, vec![1,3,5,6,8,9]),

            // Two-field combinations
            (QueryPasswordEntry { username: Some("alice"), title: Some(Some("Bank")),        ..Default::default() }, vec![0,1]),
            (QueryPasswordEntry { username: Some("alice"), title: Some(None),                ..Default::default() }, vec![2]),
            (QueryPasswordEntry { username: Some("alice"), url: Some(Some("https://bank.com")), ..Default::default() }, vec![0,2]),
            (QueryPasswordEntry { username: Some("bob"),   notes: Some(Some("note1")),       ..Default::default() }, vec![4]),
            (QueryPasswordEntry { username: Some("bob"),   notes: Some(None),                ..Default::default() }, vec![3]),
            (QueryPasswordEntry { title: Some(Some("Bank")), url: Some(Some("https://bank.com")), ..Default::default() }, vec![0,3]),
            (QueryPasswordEntry { title: Some(None),       url: Some(None),                  ..Default::default() }, vec![4,6,9]),

            // Three-field combinations
            (QueryPasswordEntry { username: Some("alice"), title: Some(Some("Bank")), url: Some(Some("https://bank.com")), ..Default::default() }, vec![0]),
            (QueryPasswordEntry { username: Some("alice"), title: Some(Some("Bank")), url: Some(Some("https://other.com")), ..Default::default() }, vec![1]),
            (QueryPasswordEntry { username: Some("dave"),  title: Some(Some("Bank")), notes: Some(Some("note1")),          ..Default::default() }, vec![7]),
            (QueryPasswordEntry { title: Some(Some("Bank")), url: Some(Some("https://bank.com")), notes: Some(None),       ..Default::default() }, vec![3]),

            // Full match (all fields specified — should return exactly one)
            (QueryPasswordEntry {
                id: None,
                username: Some("alice"),
                title: Some(Some("Bank")),
                url: Some(Some("https://bank.com")),
                notes: Some(Some("note1")),
            }, vec![0]),

            // Full match on an entry with all-None optionals
            (QueryPasswordEntry {
                id: None,
                username: Some("carol"),
                title: Some(None),
                url: Some(None),
                notes: Some(None),
            }, vec![6]),

            // No constraints — all entries returned
            (QueryPasswordEntry::default(), vec![0,1,2,3,4,5,6,7,8,9]),
        ];

        for (query, expected_indices) in cases {
            let expected_ids: Vec<&EntryId> = expected_indices.iter()
                .map(|&i| entries[i].id())
                .collect();

            let result = query.query(&root.as_ref());
            let result_ids: Vec<&EntryId> = result.iter().map(|e| e.id()).collect();

            // Order-independent comparison
            assert_eq!(
                result_ids.len(), expected_ids.len(),
                "wrong count for query {query:?}: got {:?}, expected indices {expected_indices:?}",
                result_ids
            );
            for id in &expected_ids {
                assert!(
                    result_ids.contains(id),
                    "expected id {id:?} missing from result for query {query:?}"
                );
            }
        }
    }

    #[test]
    fn delete_returns_too_many_when_multiple_match() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        for _ in 0..3 {
            InsertPasswordEntry {
                username: "duplicate".to_string(),
                password: random_string(10),
                title: None,
                url: None,
                notes: None,
            }.insert(&mut root);
        }

        let resp = QueryPasswordEntry {
            id: None,
            username: Some("duplicate"),
            ..Default::default()
        }.delete(&mut root);

        assert!(matches!(resp, DeleteResponse::TooMany(3)));

        // Entries should be untouched after a TooMany response
        let still_there = QueryPasswordEntry {
            id: None,
            username: Some("duplicate"),
            ..Default::default()
        }.query(&root.as_ref());
        assert_eq!(still_there.len(), 3);
    }
}
