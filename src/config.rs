use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the ScyllaDB connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// List of ScyllaDB nodes to connect to.
    pub hosts: Vec<String>,
    /// Port to connect on.
    pub port: u16,
    /// Keyspace to use.
    pub keyspace: String,
    /// Username for authentication, if required.
    pub username: Option<String>,
    /// Password for authentication, if required.
    pub password: Option<String>,
    /// Connection timeout.
    pub connection_timeout: Option<Duration>,
    /// Whether to use SSL/TLS.
    pub use_ssl: bool,
    /// Path to the CA certificate, if using SSL/TLS.
    pub ca_cert: Option<String>,    
}

impl Config {
    /// Creates a new `Config` instance with default settings.
    pub fn new(hosts: Vec<String>, port: u16, keyspace: String) -> Self {
        Config {
            hosts,
            port,
            keyspace,
            username: None,
            password: None,
            connection_timeout: Some(Duration::from_secs(60)),
            use_ssl: false,
            ca_cert: None,
        }
    }

    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    pub fn with_auth_and_cert(mut self, username: String, password: String, cert: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self.use_ssl = true;
        self.ca_cert = Some(cert);
        self
    }

    pub fn with_ca_cert(mut self, cert: String) -> Self {
        self.use_ssl = true;
        self.ca_cert = Some(cert);
        self
    }

    /// Sets the connection timeout.
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    /// Enables or disables SSL/TLS.
    pub fn with_ssl(mut self, use_ssl: bool) -> Self {
        self.use_ssl = use_ssl;
        self
    }

    // Add methods to adjust other configurations as needed.
}
