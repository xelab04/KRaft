use crate::Models::Cluster::ClusterResourceConfig;

#[derive(Clone)]
pub struct AppConfig {
    pub environment: String,
    pub host: String,
    pub ntfy: Option<NtfyConfig>,
    pub mail_verification: bool,
    pub email: Option<MailConfig>,
    pub jwt_secret: String,
    pub resource_config: ClusterResourceConfig,
    pub network_config: NetworkingConfig,
    pub towonel_config: Option<TowonelConfig>,
}

#[derive(Clone, Debug)]
pub struct NetworkingConfig {
    pub ingress_class: String,
    pub cluster_issuer: String,
}

#[derive(Clone, Debug)]
pub struct NtfyConfig {
    pub host: String,
    pub basic_auth: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TowonelConfig {
    pub token: String,
    pub hub: String,
}

#[derive(Clone)]
pub struct MailConfig {
    pub mail_encryption: String,
    pub mail_from_address: String,
    pub mail_from_name: String,
    pub mail_host: String,
    pub mail_mailer: String,
    pub mail_port: String,
    pub mail_password: Option<String>,
    pub mail_username: Option<String>,
}
