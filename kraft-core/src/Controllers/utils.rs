use log::{info, warn};
use regex::Regex;
use reqwest;

use crate::Models::{
    Cluster::ClusterResourceConfig,
    Config::{AppConfig, MailConfig, NetworkingConfig, NtfyConfig},
};

use kube::{
    Client,
    api::{Api, PostParams},
    core::{ApiResource, DynamicObject, GroupVersionKind},
};
use serde_json::json;

use std::env;

use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::Salt};
use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use rand::rngs::OsRng;

pub fn generate_email_config() -> Option<MailConfig> {
    let email_config = MailConfig {
        mail_encryption: env::var("MAIL_ENCRYPTION").ok()?,
        mail_from_address: env::var("MAIL_FROM_ADDRESS").ok()?,
        mail_from_name: env::var("MAIL_FROM_NAME").ok()?,
        mail_host: env::var("MAIL_HOST").ok()?,
        mail_mailer: env::var("MAIL_MAILER").ok()?,
        mail_port: env::var("MAIL_PORT").ok()?,
        mail_username: env::var("MAIL_USERNAME").ok(),
        mail_password: env::var("MAIL_PASSWORD").ok(),
    };

    Some(email_config)
}

pub fn get_ntfy_config() -> Option<NtfyConfig> {
    let host = std::env::var("NTFY_HOST").ok()?;
    let basic_auth = std::env::var("NTFY_BASIC_AUTH").ok();
    let token = std::env::var("NTFY_TOKEN").ok();

    Some(NtfyConfig {
        host,
        basic_auth,
        token,
    })
}

pub fn generate_appconfig() -> AppConfig {
    let email = generate_email_config();
    let host = std::env::var("HOST").unwrap_or_else(|_| {
        warn!("HOST not specified, defaulting to kraftcloud.dev");
        "kraftcloud.dev".to_string()
    });
    let mail_verification: bool = std::env::var("MAIL_VERIFICATION")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);
    let ntfy = get_ntfy_config();
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| {
        warn!("ENVIRONMENT not specified, defaulting to prod");
        "PROD".to_string()
    });
    let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment variables");
    let ingress_class = std::env::var("INGRESS_CLASS").unwrap_or_else(|_| {
        warn!("INGRESS_CLASS not specified, defaulting to traefik");
        String::from("traefik")
    });
    let cluster_issuer =
        std::env::var("CLUSTER_ISSUER").expect("CLUSTER_ISSUER not set in environment variables");

    let f = std::fs::File::open("/config/resourceconfig.yaml")
        .expect("Could not open /config/resourceconfig.yaml");
    let resource_config: ClusterResourceConfig =
        serde_yaml::from_reader(f).expect("Invalid yaml in /config/resourceconfig.yaml");

    let network_config = NetworkingConfig {
        ingress_class,
        cluster_issuer,
    };

    let conf: AppConfig = AppConfig {
        email,
        host,
        mail_verification,
        environment,
        ntfy,
        jwt_secret,
        resource_config,
        network_config,
    };

    conf
}

pub async fn validate_tlssan(tlssan: String) -> Result<bool, String> {
    if !tlssan.is_ascii() {
        return Err("Invalid URL".to_string());
    }

    let domain_pattern = r"^([A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)+[A-Za-z]{2,63}$";
    let re = Regex::new(domain_pattern).unwrap();

    if !re.is_match(&tlssan) {
        return Err("Malformed URL".to_string());
    }

    Ok(true)
}

pub fn panic_ntfy(config: &NtfyConfig, message: &str, title: &str) {
    let client = reqwest::blocking::Client::new();

    let mut request = client
        .post(&config.host)
        .header("Title", title)
        .body(message.to_string());

    if let Some(auth) = &config.basic_auth {
        request = request.header("Authorization", format!("Basic {auth}"));
    }
    if let Some(auth) = &config.token {
        request = request.header("Authorization", format!("Bearer {auth}"));
    }

    match request.send() {
        Ok(r) => match r.error_for_status() {
            Ok(_) => {
                info!("Ntfy panic message sent");
            }
            Err(e) => {
                info!("Error message; {}", e);
            }
        },
        Err(_) => {
            info!("Error sending ntfy panic message, ironic")
        }
    }
}

pub async fn send_ntfy_notif(
    host: &str,
    message: &str,
    title: &str,
    basic_auth: &Option<String>,
    token: &Option<String>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let mut request = client
        .post(host)
        .header("Title", title)
        .body(message.to_string());

    if let Some(auth) = basic_auth {
        request = request.header("Authorization", format!("Basic {auth}"));
    }
    if let Some(auth) = token {
        request = request.header("Authorization", format!("Bearer {auth}"));
    }

    let r = request.send().await.unwrap();

    info!("{:?}", r);

    match r.error_for_status() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn namevalid(name: &str) -> bool {
    name.chars()
        .all(|ch: char| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

pub fn check_passwords_match(clear_pwd: &String, hashed: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hashed) {
        Ok(hash) => hash,
        Err(_e) => {
            return false;
        }
    };

    Argon2::default()
        .verify_password(clear_pwd.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn hash_password(clear_pwd: &String) -> String {
    let salt_str = &SaltString::generate(&mut OsRng);
    let salt: Salt = salt_str.into();

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(clear_pwd.as_bytes(), salt)
        .expect("Error hashing password.");

    password_hash.to_string()
}

pub async fn send_mail(
    mail_config: &MailConfig,
    recipient_email: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let mail = MessageBuilder::new()
        .from((
            mail_config.mail_from_name.as_str(),
            mail_config.mail_from_address.as_str(),
        ))
        .to(recipient_email)
        .subject(subject)
        .text_body(body);

    let int_port: u16 = mail_config
        .mail_port
        .parse()
        .expect("Port should be an integer value");

    let mut smtp_client = SmtpClientBuilder::new(mail_config.mail_host.as_str(), int_port).unwrap();
    if mail_config.mail_encryption == "tls" {
        smtp_client = smtp_client.implicit_tls(false);
    }
    if let (Some(username), Some(password)) =
        (&mail_config.mail_username, &mail_config.mail_password)
    {
        smtp_client = smtp_client.credentials((username.as_str(), password.as_str()));
    }

    smtp_client
        .connect()
        .await
        .unwrap()
        .send(mail)
        .await
        .unwrap();

    Ok(())
}

pub fn convert_cpu(value: &str) -> i32 {
    let r: i32;
    if value.ends_with("n") {
        r = value
            .strip_suffix("n")
            .unwrap()
            .parse::<i32>()
            .unwrap_or(i32::MAX)
            / 1000000;
    } else if value.ends_with("m") {
        r = value.strip_suffix("m").unwrap().parse::<i32>().unwrap();
    } else {
        r = value.parse::<i32>().unwrap() * 1000;
    }
    r
}

pub fn convert_memory(value: &str) -> i32 {
    let r: i32;
    if value.ends_with("Ki") {
        r = value.strip_suffix("Ki").unwrap().parse::<i32>().unwrap() / 1000;
    } else if value.ends_with("Mi") {
        r = value.strip_suffix("Mi").unwrap().parse::<i32>().unwrap();
    } else if value.ends_with("Gi") {
        r = value.strip_suffix("Gi").unwrap().parse::<i32>().unwrap() * 1000;
    } else {
        r = value.parse::<i32>().unwrap() / 1000000;
    }
    r
}
