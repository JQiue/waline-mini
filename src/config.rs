//! config

use serde::{de::Deserializer, Deserialize};

use crate::error::AppError;

fn default_ipqps() -> u64 {
  60
}

fn default_akismet_key() -> String {
  "86fe49f5ea50".to_string()
}

fn default_login() -> String {
  "no".to_string()
}

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

fn default_false() -> bool {
  false
}

#[derive(Deserialize)]
pub struct EnvConfig {
  pub database_url: String,
  pub jwt_token: String,
  pub site_name: String,
  pub site_url: String,
  pub smtp_service: Option<String>,
  pub smtp_host: Option<String>,
  pub smtp_port: Option<u16>,
  pub smtp_user: Option<String>,
  pub smtp_pass: Option<String>,
  pub author_email: Option<String>,
  pub levels: Option<String>,
  #[serde(default = "default_ipqps")]
  pub ipqps: u64,
  #[serde(default = "default_false")]
  pub comment_audit: bool,
  #[serde(default = "default_akismet_key")]
  pub akismet_key: String,
  #[serde(default = "default_login")]
  pub login: String,
  #[serde(default = "default_false")]
  pub disable_author_notify: bool,
  #[serde(default, deserialize_with = "deserialize_comma_separated")]
  pub disallow_ip_list: Vec<String>,
  #[serde(default, deserialize_with = "deserialize_comma_separated")]
  pub forbidden_words: Vec<String>,
  #[serde(default, deserialize_with = "deserialize_comma_separated")]
  pub secure_domians: Vec<String>,
  #[serde(default = "default_false")]
  pub disable_useragent: bool,
  #[serde(default = "default_false")]
  pub disable_region: bool,
  pub ip2region_db: Option<String>,
}

impl EnvConfig {
  pub fn load_env() -> Result<EnvConfig, AppError> {
    dotenvy::from_filename_override(".shuttle.env").ok();
    dotenvy::from_filename_override(".env").ok();
    Ok(envy::from_env::<EnvConfig>()?)
  }
}
