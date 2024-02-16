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
    // Add additional parameters as needed.
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
            connection_timeout: Some(Duration::from_secs(5)),
            use_ssl: false,
        }
    }

    /// Sets the authentication credentials.
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
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
