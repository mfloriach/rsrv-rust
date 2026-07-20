use anyhow::{Result, anyhow, bail};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::Error,
    password_hash::{SaltString, rand_core::OsRng},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

static SECRET: &str = "secret";

#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    exp: usize,
    sub: String,
    email: String,
}

// Hash the given password using Argon2 and return the hashed password as a string
pub fn hash_password(password: &String) -> Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("could not hashed the password: {e}"))?;

    Ok(hashed_password.to_string())
}

// Generate a JWT token for the given username
pub fn generate_token(email: String, sub: String) -> Result<String> {
    let claims = Claims {
        sub,
        email,
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize
            + 60 * 60 * 24,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET.as_bytes()))
        .map_err(|e| anyhow!("could not encode token: {e}"))?;

    Ok(token)
}

// Verify the given password against the hashed password
pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|e| anyhow!("{e}"))?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(Error::Password) => Ok(false),
        Err(e) => Err(anyhow!("{e}")),
    }
}

// Verify the given JWT token and return the subject (sub) if valid
pub fn verify_token(token: &str) -> Result<String> {
    match decode(token, &DecodingKey::from_secret(SECRET.as_bytes()), &Validation::default()) {
        Ok(token_data) => {
            let claims: Claims = token_data.claims;
            Ok(claims.sub)
        }
        Err(err) => {
            bail!("Token verification failed: {}", err);
        }
    }
}
