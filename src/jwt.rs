use anyhow::{Result, anyhow, bail};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct JwtConfig {
    secret: SecretString,
    expiration_seconds: u64,
}

static JWT_CONFIG: OnceLock<Result<JwtConfig, String>> = OnceLock::new();

/// Caches the validated JWT secret. Call this during application startup.
pub fn initialize_jwt_config(secret: SecretString, expiration_seconds: u64) -> Result<()> {
    if expiration_seconds == 0 {
        bail!("JWT expiration must be greater than zero");
    }

    match JWT_CONFIG.get_or_init(|| Ok(JwtConfig { secret, expiration_seconds })) {
        Ok(config) => Ok(config),
        Err(error) => Err(anyhow!(error.clone())),
    }
    .map(|_| ())
}

fn jwt_config() -> Result<&'static JwtConfig> {
    match JWT_CONFIG.get() {
        Some(Ok(config)) => Ok(config),
        Some(Err(error)) => Err(anyhow!(error.clone())),
        None => Err(anyhow!("JWT secret has not been initialized")),
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    exp: usize,
    sub: uuid::Uuid,
    email: String,
}

// Generate a JWT token for the given username
pub fn generate_token(email: String, sub: uuid::Uuid) -> Result<String> {
    let config = jwt_config()?;
    let claims = Claims {
        sub,
        email: email.to_string(),
        exp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize
            + config.expiration_seconds as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.expose_secret().as_bytes()),
    )
    .map_err(|e| anyhow!("could not encode token: {e}"))?;

    Ok(token)
}

// Verify the given JWT token and return the subject (sub) if valid
pub fn verify_token(token: &str) -> Result<uuid::Uuid> {
    let config = jwt_config()?;

    match decode(
        token,
        &DecodingKey::from_secret(config.secret.expose_secret().as_bytes()),
        &Validation::default(),
    ) {
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
    use super::{
        Claims, DecodingKey, Validation, decode, generate_token, initialize_jwt_config, jwt_config,
        verify_token,
    };
    use secrecy::{ExposeSecret, SecretString};

    fn initialize_test_secret() {
        initialize_jwt_config(SecretString::new("test-secret".to_owned()), 86_400)
            .expect("JWT secret should be initialized");
    }

    #[test]
    fn generated_token_contains_the_expected_claims() {
        initialize_test_secret();
        let email = "user@example.com".to_string();
        let subject = uuid::Uuid::now_v7();
        let token = generate_token(email.clone(), subject).expect("token should be generated");

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(
                jwt_config()
                    .expect("JWT configuration should be initialized")
                    .secret
                    .expose_secret()
                    .as_bytes(),
            ),
            &Validation::default(),
        )
        .expect("token should be valid");

        assert_eq!(token_data.claims.email, email);
        assert_eq!(token_data.claims.sub, subject);
    }

    #[test]
    fn verifies_a_generated_token_subject() {
        initialize_test_secret();
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
