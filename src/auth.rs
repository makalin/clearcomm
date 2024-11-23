use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jwt::{Header, Token};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

pub struct Auth {
    secret_key: Vec<u8>,
}

impl Auth {
    pub fn new(secret_key: Vec<u8>) -> Self {
        Self { secret_key }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn generate_token(&self, username: &str) -> Result<String> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            + 24 * 3600; // 24 hour expiration

        let claims = Claims {
            sub: username.to_string(),
            exp: expiration,
        };

        let header = Header::default();
        let token = Token::new(header, claims);
        Ok(token.sign(&self.secret_key)?)
    }

    pub fn verify_token(&self, token: &str) -> Result<String> {
        let token: Token<Header, Claims, _> = Token::verify(&self.secret_key, token)?;
        let claims = token.claims();
        
        if claims.exp < SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() {
            return Err("Token expired".into());
        }

        Ok(claims.sub.clone())
    }
}