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

#[cfg(test)]
mod tests {
    use super::{Claims, DecodingKey, SECRET, Validation, decode, generate_token, verify_token};

    #[test]
    fn generated_token_contains_the_expected_claims() {
        let email = "user@example.com".to_string();
        let subject = uuid::Uuid::now_v7();
        let token = generate_token(email.clone(), subject).expect("token should be generated");

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_bytes()),
            &Validation::default(),
        )
        .expect("token should be valid");

        assert_eq!(token_data.claims.email, email);
        assert_eq!(token_data.claims.sub, subject);
    }

    #[test]
    fn verifies_a_generated_token_subject() {
        let subject = uuid::Uuid::now_v7();
        let token = generate_token("user@example.com".to_string(), subject)
            .expect("token should be generated");

        assert_eq!(verify_token(&token).expect("token should be valid"), subject);
    }

    #[test]
    fn rejects_an_invalid_token() {
        assert!(verify_token("not-a-jwt").is_err());
    }
}
