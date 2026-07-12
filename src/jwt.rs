use anyhow::{Result, bail};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
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

    let hashed_password = argon2.hash_password(password.as_bytes(), &salt).expect("sdfdfdsds");

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

    Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET.as_bytes()))
        .expect("could not encode"))
}

// Verify the given password against the hashed password
pub fn verify_password(password: &String, hashed_password: &str) -> Result<()> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hashed_password).unwrap();

    argon2.verify_password(password.as_bytes(), &parsed_hash).expect("dfd");
    Ok(())
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
