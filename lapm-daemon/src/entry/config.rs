use keepass::db::{GroupMut, fields};


use super::EntryError;

#[derive(Debug,Clone)]
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

#[cfg(test)]
mod tests {
    use keepass::Database;
    use rand::RngExt;

    use super::ConfigEntry;


    impl PartialEq for ConfigEntry {
        fn eq(&self, other: &Self) -> bool {
            (self.key.as_str() == other.key.as_str()) &&
            (self.value.as_str() == other.value.as_str())
        }
    }

    impl Eq for ConfigEntry {}



    fn generate_random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();

        (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }


    #[test]
    fn set_entry_can_be_retrieved() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let cases = (0..25).map(|_| ConfigEntry {
            key: generate_random_string(15),
            value: generate_random_string(25),
        }).collect::<Vec<_>>();

        for case in &cases {
            case.clone().set(&mut root);
        }

        for case in &cases {
            assert_eq!(
                Some(case),
                ConfigEntry::get(&mut root, case.key.clone()).as_ref()
            );
        }
    }

    #[test]
    fn set_updates_existing() {
        let mut db = Database::new();
        let mut root = db.root_mut();

        let key = "key".to_string();

        ConfigEntry {
            key: key.clone(),
            value: "old".to_string(),
        }.set(&mut root);

        let new = ConfigEntry {
            key: key.clone(),
            value: "new".to_string(),
        };
        new.clone().set(&mut root);

        assert_eq!(Some(new), ConfigEntry::get(&mut root, key.clone()));
    }

    #[test]
    fn bool_serialization_is_symmetric() {
        let true_ent = ConfigEntry::from_bool("key".to_string(), true);
        assert_eq!(true_ent.to_bool().ok(), Some(true));
        let false_ent = ConfigEntry::from_bool("key".to_string(), false);
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
