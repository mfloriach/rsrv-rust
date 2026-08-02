use anyhow::{Result, anyhow};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::Error,
    password_hash::{SaltString, rand_core::OsRng},
};

// Hash the given password using Argon2 and return the hashed password as a string
pub fn hash_password(password: impl Into<String>) -> Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = argon2
        .hash_password(password.into().as_bytes(), &salt)
        .map_err(|e| anyhow!("could not hashed the password: {e}"))?;

    Ok(hashed_password.to_string())
}

// Verify the given password against the hashed password
pub fn verify_password(password: impl Into<String>, hashed_password: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|e| anyhow!("{e}"))?;

    match Argon2::default().verify_password(password.into().as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(Error::Password) => Ok(false),
        Err(e) => Err(anyhow!("{e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn verifies_the_password_used_to_create_the_hash() {
        let password = "correct-horse-battery-staple".to_string();
        let hash = hash_password(&password).expect("password should be hashed");

        assert!(verify_password(&password, &hash).expect("hash should be valid"));
    }

    #[test]
    fn rejects_an_incorrect_password() {
        let password = "correct-horse-battery-staple".to_string();
        let hash = hash_password(&password).expect("password should be hashed");

        assert!(!verify_password("wrong-password", &hash).expect("hash should be valid"));
    }
}
