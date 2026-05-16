use thiserror::Error;

mod password;
pub use password::PasswordEntry;

mod config;


#[derive(Debug,Clone,Error)]
pub enum EntryError {
    #[error("Can't serialize {value} as {value_t}")]
    SerializationError {
        value: String,
        value_t: &'static str,
    },
}
