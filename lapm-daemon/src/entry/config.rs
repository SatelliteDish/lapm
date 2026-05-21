use keepass::db::EntryRef;
use keepass::db::{GroupMut, fields};

use std::time::Duration;
use crate::entry::{FieldError, Insert, Update};
use super::{
    Query,
    EntryError,
    DeleteResponse,
};

#[derive(Debug,Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

impl ConfigEntry {
    pub fn to_bool(self) -> Result<bool, EntryError> {
        let value = self.value.as_str();
        if value == "true" {
            Ok(true)
        } else if value == "false" {
            Ok(false)
        } else {
            Err(EntryError::SerializationError { value: self.value, value_t: "bool" })
        }
    }

    pub fn to_duration(self) -> Result<Duration, EntryError> {
        let secs = self.value.parse::<u64>()
            .map_err(|_| EntryError::SerializationError {
                value: self.value,
                value_t: "u64",
            })?;
        Ok(Duration::new(secs,0))
    }
}

impl TryFrom<&EntryRef<'_>> for ConfigEntry {
    type Error = EntryError;

    fn try_from(value: &EntryRef<'_>) -> Result<Self, Self::Error> {
        let key = value.get(fields::TITLE);
        let value = value.get("value");

        if let (Some(key), Some(value)) = (key,value) {
            Ok(Self { key: key.to_string(), value: value.to_string() })
        } else {
            let mut errs: Vec<FieldError> = vec![];

            if key.is_none() {
                errs.push(FieldError { name: "key", reason: "is required".to_string() });
            }
            if value.is_none() {
                errs.push(FieldError { name: "value", reason: "is required".to_string() });
            }

            Err(EntryError::InvalidFields { fields: errs })
        }
   }
}

impl TryFrom<EntryRef<'_>> for ConfigEntry {
    type Error = EntryError;

    fn try_from(value: EntryRef<'_>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

#[derive(Debug,Clone)]
pub struct QueryConfigEntry<'q>{
    pub key: Option<&'q str>,
    pub value: Option<&'q str>,
}

impl Default for QueryConfigEntry<'_> {
    fn default() -> Self {
        Self {
            key: None,
            value: None,
        }
    }
}

impl PartialEq<ConfigEntry> for QueryConfigEntry<'_> {
    fn eq(&self, other: &ConfigEntry) -> bool {
        if let Some(key) = self.key {
            if key != other.key.as_str() {
                return false;
            }
        }

        if let Some(value) = self.value {
            if value != other.value.as_str() {
                return false;
            }
        }

        true
    }
}

impl PartialEq<EntryRef<'_>> for QueryConfigEntry<'_> {
    fn eq(&self, other: &EntryRef<'_>) -> bool {
        let entry = match ConfigEntry::try_from(other) {
            Ok(ent) => ent,
            Err(_) => return false,
        };
        if let Some(key) = self.key {
            if key != entry.key.as_str() {
                return false;
            }
        }

        if let Some(value) = self.value {
            if value != entry.value.as_str() {
                return false;
            }
        }

        true
    }
}

impl Query for QueryConfigEntry<'_> {
    type Response = ConfigEntry;

    fn query(&self, group: &keepass::db::GroupRef<'_>) -> Vec<Self::Response> {
        let mut res: Vec<Self::Response> = vec![];

        for ent in group.entries() {
            if self == &ent {
                if let Ok(cfg) = ConfigEntry::try_from(ent) {
                    res.push(cfg);
                }
            }
        }

        res
    }

    fn delete(&self, group: & mut GroupMut<'_>) -> super::DeleteResponse {
        let found = self.query(&group.as_ref());
        match found.len() {
            0 => DeleteResponse::NotFound,
            1 => {
                group.entry_by_name_mut(found[0].key.as_str())
                    .unwrap().remove(); // Can safely unwrap
                DeleteResponse::Success
            },
            _ => DeleteResponse::TooMany(found.len())
        }
    }
}

#[derive(Debug,Clone)]
pub struct InsertConfigEntry {
    pub key: String,
    pub value: String,
}

impl InsertConfigEntry {
    pub fn from_bool(key: String, value: bool) -> Self {
        Self {
            key,
            value: value.to_string(),
        }
    }

    pub fn from_duration(key: String, value: Duration) -> Self {
        Self {
            key,
            value: value.as_secs().to_string(),
        }
    }
}

impl Insert for InsertConfigEntry {
    type Response = ConfigEntry;

    fn insert(self, group: &mut GroupMut<'_>) -> Self::Response {
        let mut ent = group.add_entry();
        ent.set_unprotected(fields::TITLE, &self.key);
        ent.set_unprotected("value", &self.value);

        ConfigEntry {
            key: self.key,
            value: self.value,
        }
    }
}

#[derive(Debug,Clone)]
pub struct UpdateConfigEntry<'q> {
    pub key: &'q str,
    pub value: String,
}

impl Update for UpdateConfigEntry<'_> {
    fn update(self, group: &mut GroupMut) -> Option<()> {
        let ids = group.entry_ids().collect::<Vec<_>>();
        for id in ids {
            let mut ent = group.entry_mut(id)?;
            if ent.get(fields::TITLE)? == self.key {
                ent.set_unprotected("value", &self.value);
                return Some(())
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use keepass::Database;

    use crate::test_helpers::random_string;
    use super::*;


    impl PartialEq for ConfigEntry {
        fn eq(&self, other: &Self) -> bool {
            (self.key.as_str() == other.key.as_str()) &&
            (self.value.as_str() == other.value.as_str())
        }
    }

    impl Eq for ConfigEntry {}

    #[test]
    fn set_entry_can_be_retrieved() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let cases = (0..25).map(|_| ConfigEntry {
            key: random_string(15),
            value: random_string(25),
        }).collect::<Vec<_>>();

        for case in &cases {
            let ConfigEntry { key, value } = case;

            InsertConfigEntry {
                key: key.to_string(),
                value: value.to_string(),
            }.insert(&mut root);
        }

        for case in cases {
            assert_eq!(
                vec![case.clone()],
                QueryConfigEntry{
                    key: Some(&case.key.as_str()),
                    value: None,
                }
                    .query(&root.as_ref()),
            );
        }
    }

    #[test]
    fn set_updates_existing() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let key = "key".to_string();

        InsertConfigEntry {
            key: key.clone(),
            value: "old".to_string(),
        }.insert(&mut root);

        let new = UpdateConfigEntry {
            key: &key,
            value: "new".to_string(),
        };
        new.clone().update(&mut root);

        assert_eq!(
            Some(&ConfigEntry {
                key: key.to_string(),
                value: new.value.to_string(),
            }),
            QueryConfigEntry{ key: Some(&key), value: None }
                .query(&root.as_ref())
                .get(0));
    }

    #[test]
    fn bool_serialization_is_symmetric() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let true_ent = InsertConfigEntry::from_bool("key".to_string(), true)
            .insert(&mut root);
        assert_eq!(true_ent.to_bool().ok(), Some(true));
        let false_ent = InsertConfigEntry::from_bool("key".to_string(), false)
            .insert(&mut root);
        assert_eq!(false_ent.to_bool().ok(), Some(false));
    }

    #[test]
    fn unexpected_values_fail_to_parse_to_bool() {
        let ent = ConfigEntry {
            key: "key".to_string(),
            value: "This is definitely not a bool!".to_string(),
        };
        assert!(ent.to_bool().is_err());
    }
}
