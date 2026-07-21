use anyhow::{Result, anyhow, bail};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

static SECRET: &str = "secret";

#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    exp: usize,
    sub: uuid::Uuid,
    email: String,
}

// Generate a JWT token for the given username
pub fn generate_token(email: String, sub: uuid::Uuid) -> Result<String> {
    let claims = Claims {
        sub,
        email: email.to_string(),
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize
            + 60 * 60 * 24,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET.as_bytes()))
        .map_err(|e| anyhow!("could not encode token: {e}"))?;

    Ok(token)
}

// Verify the given JWT token and return the subject (sub) if valid
pub fn verify_token(token: &str) -> Result<uuid::Uuid> {
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
