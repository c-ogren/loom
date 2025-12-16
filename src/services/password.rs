use argon2::password_hash::{Error as PasswordHashError, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use core::result::Result::Ok;
use rand::rngs::OsRng;
use std::str;

// Hash a password during client registration
pub fn hash_password(plain: &str) -> anyhow::Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_hash(plain: &[u8], hash: &str) -> bool {
    let parsed = PasswordHash::new(hash);
    parsed
        .and_then(|ph| Argon2::default().verify_password(plain, &ph))
        .is_ok()
}
