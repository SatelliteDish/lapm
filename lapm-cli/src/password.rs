use thiserror::Error;

#[derive(Debug,Error)]
pub enum PasswordError {
    #[error("IO failure while reading password")]
    IOError(#[from]std::io::Error),
    #[error("Invalid password entered")]
    InvalidEntry,
    #[error("Failed to confirm password in {tries} tries")]
    FailedConfirmation{ tries: u8 }
}

pub fn prompt_and_validate_password(prompt: impl std::fmt::Display, tries: u8, validator: impl Fn(&str) -> bool) -> Result<String,PasswordError> {
    for _ in 0..tries {
        println!("{prompt}");
        let password = rpassword::read_password()?;
        if validator(&password) {
            return Ok(password);
        }
    }

    Err(PasswordError::InvalidEntry)
}

pub fn prompt_password(prompt: impl std::fmt::Display, tries: u8) -> Result<String, PasswordError> {
    prompt_and_validate_password(prompt, tries, |pw| !pw.is_empty())
}

pub fn get_and_confirm_password() -> Result<String, PasswordError> {
    let tries = 3_u8;
    let password = prompt_password("Please enter a new password:", tries)?;
    prompt_and_validate_password(
        "Please re-enter your password:", tries, |pw| pw == password
    ).map_err(|e| {
            if let PasswordError::InvalidEntry = e {
                PasswordError::FailedConfirmation { tries }
            } else { e }
    })
}
