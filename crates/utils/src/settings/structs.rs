use doku::Document;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct Settings {
  /// settings related to the postgresql database
  #[default(Default::default())]
  pub database: DatabaseConfig,
  /// Settings related to activitypub federation
  /// Pictrs image server configuration.
  #[default(Some(Default::default()))]
  pub(crate) pictrs: Option<PictrsConfig>,
  /// Email sending configuration. All options except login/password are mandatory
  /// Parameters for automatic configuration of new instance (only used at first start)
  #[default(None)]
  #[doku(example = "Some(Default::default())")]
  pub setup: Option<SetupConfig>,
  /// the domain name of your instance (mandatory)
  #[default("unset")]
  #[doku(example = "example.com")]
  pub hostname: String, // TODO this is duplicated in the instance / local_site table now tho?
  /// Address where lemmy should listen for incoming requests
  #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
  #[doku(as = "String")]
  pub bind: IpAddr,
  /// Port where lemmy should listen for incoming requests
  #[default(8536)]
  pub port: u16,
  /// Whether the site is available over TLS. Needs to be true for federation to work.
  #[default(true)]
  pub tls_enabled: bool,
  /// Set the URL for opentelemetry exports. If you do not have an opentelemetry collector, do not set this option
  #[default(None)]
  #[doku(skip)]
  pub opentelemetry_url: Option<Url>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct PictrsConfig {
  /// Address where pictrs is available (for image hosting)
  #[default(Url::parse("http://pictrs:8080").expect("parse pictrs url"))]
  #[doku(example = "http://pictrs:8080")]
  pub url: Url,

  /// Set a custom pictrs API key. ( Required for deleting images )
  #[default(None)]
  pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct DatabaseConfig {
  /// Username to connect to postgres
  #[default("lemmy")]
  pub(super) user: String,
  /// Password to connect to postgres
  #[default("password")]
  pub password: String,
  #[default("localhost")]
  /// Host where postgres is running
  pub host: String,
  /// Port where postgres can be accessed
  #[default(5432)]
  pub(super) port: i32,
  /// Name of the postgres database for lemmy
  #[default("lemmy")]
  pub(super) database: String,
  /// Maximum number of active sql connections
  #[default(5)]
  pub pool_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
pub struct SetupConfig {
  /// Username for the admin user
  #[doku(example = "admin")]
  pub admin_username: String,
  /// Password for the admin user. It must be at least 10 characters.
  #[doku(example = "tf6HHDS4RolWfFhk4Rq9")]
  pub admin_password: String,
  /// Name of the site (can be changed later)
  #[doku(example = "My Lemmy Instance")]
  pub site_name: String,
  /// Email for the admin user (optional, can be omitted and set later through the website)
  #[doku(example = "user@example.com")]
  #[default(None)]
  pub admin_email: Option<String>,
}
