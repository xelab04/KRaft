use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::Salt};
use argon2::{password_hash::{PasswordHash, SaltString}};

pub fn check_passwords_match(clear_pwd:&String, hashed: &String) -> bool {

    let parsed_hash = match PasswordHash::new(hashed) {
        Ok(hash) => hash,
        Err(e) => {
            return false;
        }
    };

    match Argon2::default().verify_password(clear_pwd.as_bytes(), &parsed_hash) {
        Ok(_) => { return true; }
        Err(_) => { return false; }
    }
}

pub fn hash_password(clear_pwd: &String) -> String {
    let salt_str = &SaltString::generate(&mut rand::rngs::OsRng);
    let salt: Salt = salt_str.try_into().unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(clear_pwd.as_bytes(), salt).expect("Error hashing password.");

    return password_hash.to_string();
}
