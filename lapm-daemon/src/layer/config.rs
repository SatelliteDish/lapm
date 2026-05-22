use std::time::Duration;
use keepass::{
    Database,
    db::{
        GroupMut,
        GroupRef,
    }
};
use lapm_core::command::layer::DaemonLayerConfig;

use crate::entry::{
    Insert, Query, Update, config::{InsertConfigEntry, QueryConfigEntry, UpdateConfigEntry}
};



const DEF_TIMEOUT: u64 = 600; // 10 minutes

pub const CONFIG_GROUP_NAME: &str = "__config__";
const CONFIG_PUBLIC_USER_KEY: &str = "public_usernames";
const CONFIG_TIMEOUT_KEY: &str = "timeout";

#[derive(Debug,Clone)]
pub struct LayerConfig {
    pub timeout: Option<Duration>,
    pub public_usernames: bool,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::new(DEF_TIMEOUT,0)),
            public_usernames: false,
        }
    }
}

impl From<&LayerConfig> for DaemonLayerConfig {
    fn from(value: &LayerConfig) -> Self {
        Self {
            timeout: value.timeout.map(|tmt| tmt.as_secs()),
            public_usernames: value.public_usernames,
        }
    }
}

impl From<LayerConfig> for DaemonLayerConfig {
    fn from(value: LayerConfig) -> Self {
        DaemonLayerConfig::from(&value)
    }
}

impl LayerConfig {
    fn get_by_key(conf_group: &GroupRef<'_>, key: &str) -> Option<String> {
        QueryConfigEntry {
            key: Some(key),
            ..Default::default()
        }.query(&conf_group)
            .into_iter()
            .next()
            .map(|cfg| cfg.value)
    }

    fn set_by_key(conf_group: &mut GroupMut, key: &str, value: &str) {
        let query_res = QueryConfigEntry {
            key: Some(key),
            ..Default::default()
        }.query(&conf_group.as_ref());
        let target = query_res.get(0);
        if target.is_some() {
            UpdateConfigEntry {
                key,
                value: value.to_string(),
            }.update(conf_group);
        } else {
            InsertConfigEntry {
                key: key.to_string(),
                value: value.to_string(),
            }.insert(conf_group);
        }
    }

    fn remove_by_key(conf_group: &mut GroupMut, key: &str) {
        if let Some(ent) = conf_group.entry_by_name_mut(key) {
            ent.remove();
        }
    }

    pub fn set_in_db(&self, db: &mut Database) {
        let mut root = db.root_mut();
        let mut config_group =  match root.group_by_name_mut(CONFIG_GROUP_NAME) {
            Some(gp) => gp,
            None => {
                let mut group = root.add_group();
                group.name = CONFIG_GROUP_NAME.to_string();
                group
            },
        };

        Self::set_by_key(
            &mut config_group,
            CONFIG_PUBLIC_USER_KEY,
            &self.public_usernames.to_string(),
        );

        match self.timeout {
            Some(timeout) => Self::set_by_key(
                &mut config_group,
                CONFIG_TIMEOUT_KEY,
                &timeout.as_secs().to_string(),
            ),
            None => Self::remove_by_key(&mut config_group, CONFIG_TIMEOUT_KEY),
        }
    }

    pub fn from_db(db: &mut Database) -> Option<Self> {
        let root = db.root();
        let config_group = root.group_by_name(CONFIG_GROUP_NAME)?;

        Some(Self {
            public_usernames: Self::get_by_key(&config_group, CONFIG_PUBLIC_USER_KEY)
                .map(|str| if str == "true" { true } else { false })
                .unwrap_or(false),
            timeout: Self::get_by_key(&config_group, CONFIG_TIMEOUT_KEY)
                .map(|st| Duration::new(st.parse::<u64>().unwrap_or(60),0)),
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    impl PartialEq<LayerConfig> for LayerConfig {
        fn eq(&self, other: &LayerConfig) -> bool {
            (self.timeout == other.timeout) &&
            (self.public_usernames == other.public_usernames)
        }
    }

    use super::*;
    use crate::test_helpers::random_opt;

    fn get_test_configs(count: usize) -> Vec<LayerConfig> {
        let mut rng = rand::rng();
        (0..count).map(|_| LayerConfig {
            timeout: random_opt(Duration::new(rng.random_range(60..10_000),0)),
            public_usernames: rng.random_bool(1.0/2.0),
        }).collect()
    }

    #[test]
    fn set_and_get_in_is_db_symmetrical() {
        let mut db = Database::new();
        for conf in get_test_configs(25).into_iter() {
            conf.set_in_db(&mut db);
            assert_eq!(Some(conf), LayerConfig::from_db(&mut db));
        }
    }
}
