use std::env;

use actix_web::App;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::Salt};
use argon2::{password_hash::{PasswordHash, SaltString}};
use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;

use crate::class::{MailConfig, AppConfig};


pub fn generate_email_config() -> Option<MailConfig> {
    let email_config = MailConfig{
        mail_encryption: env::var("MAIL_ENCRYPTION").ok()?,
        mail_from_address: env::var("MAIL_FROM_ADDRESS").ok()?,
        mail_from_name: env::var("MAIL_FROM_NAME").ok()?,
        mail_host: env::var("MAIL_HOST").ok()?,
        mail_mailer: env::var("MAIL_HOST").ok()?,
        mail_port: env::var("MAIL_HOST").ok()?,
        mail_username: env::var("MAIL_HOST").ok(),
        mail_password: env::var("MAIL_PASSWORD").ok()
    };

    Some(email_config)
}

pub fn generate_appconfig() -> AppConfig {
    let email_config = generate_email_config();
    let host = std::env::var("HOST").unwrap_or_else(|_| "kraftcloud.dev".to_string());

    let conf: AppConfig = AppConfig {
        email: email_config,
        host: host
    };

    conf
}



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

pub async fn send_mail(mail_config: &MailConfig, recipient_email: &str, subject: &str, body: &str) -> Result<(), String> {

    let mail = MessageBuilder::new()
        .from((mail_config.mail_from_name.as_str(), mail_config.mail_from_address.as_str()))
        .to(recipient_email)
        .subject(subject)
        .text_body(body);

    let int_port: u16 = mail_config.mail_port.parse().expect("Port should be an integer value");

    let mut smtp_client = SmtpClientBuilder::new(mail_config.mail_host.as_str(), int_port);
    if mail_config.mail_encryption == "tls" {
        smtp_client = smtp_client.implicit_tls(false);
    }
    if let (Some(username), Some(password)) = (&mail_config.mail_username,&mail_config.mail_password) {
        smtp_client = smtp_client.credentials((username.as_str(), password.as_str()));
    }

    smtp_client.connect()
        .await
        .unwrap()
        .send(mail)
        .await
        .unwrap();

    Ok(())
}
