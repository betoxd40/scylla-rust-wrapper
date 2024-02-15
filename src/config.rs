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

/// Possible methods of how a connection is recycled.
///
/// The default is [`Fast`] which does not check the connection health or
/// perform any clean-up queries.
///
/// [`Fast`]: RecyclingMethod::Fast
/// [`Verified`]: RecyclingMethod::Verified
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum RecyclingMethod {
    /// Only run [`Client::is_closed()`][1] when recycling existing connections.
    ///
    /// Unless you have special needs this is a safe choice.
    ///
    /// [1]: tokio_postgres::Client::is_closed
    Fast,

    /// Run [`Client::is_closed()`][1] and execute a test query.
    ///
    /// This is slower, but guarantees that the database connection is ready to
    /// be used. Normally, [`Client::is_closed()`][1] should be enough to filter
    /// out bad connections, but under some circumstances (i.e. hard-closed
    /// network connections) it's possible that [`Client::is_closed()`][1]
    /// returns `false` while the connection is dead. You will receive an error
    /// on your first query then.
    ///
    /// [1]: tokio_postgres::Client::is_closed
    Verified,

    /// Like [`Verified`] query method, but instead use the following sequence
    /// of statements which guarantees a pristine connection:
    /// ```sql
    /// CLOSE ALL;
    /// SET SESSION AUTHORIZATION DEFAULT;
    /// RESET ALL;
    /// UNLISTEN *;
    /// SELECT pg_advisory_unlock_all();
    /// DISCARD TEMP;
    /// DISCARD SEQUENCES;
    /// ```
    ///
    /// This is similar to calling `DISCARD ALL`. but doesn't call
    /// `DEALLOCATE ALL` and `DISCARD PLAN`, so that the statement cache is not
    /// rendered ineffective.
    ///
    /// [`Verified`]: RecyclingMethod::Verified
    Clean,

    /// Like [`Verified`] but allows to specify a custom SQL to be executed.
    ///
    /// [`Verified`]: RecyclingMethod::Verified
    Custom(String),
}

impl Default for RecyclingMethod {
    fn default() -> Self {
        Self::Fast
    }
}

impl RecyclingMethod {
    const DISCARD_SQL: &'static str = "\
        CLOSE ALL; \
        SET SESSION AUTHORIZATION DEFAULT; \
        RESET ALL; \
        UNLISTEN *; \
        SELECT pg_advisory_unlock_all(); \
        DISCARD TEMP; \
        DISCARD SEQUENCES;\
    ";

    /// Returns SQL query to be executed when recycling a connection.
    pub fn query(&self) -> Option<&str> {
        match self {
            Self::Fast => None,
            Self::Verified => Some(""),
            Self::Clean => Some(Self::DISCARD_SQL),
            Self::Custom(sql) => Some(sql),
        }
    }
}

/// Configuration object for a [`Manager`].
///
/// This currently only makes it possible to specify which [`RecyclingMethod`]
/// should be used when retrieving existing objects from the [`Pool`].
///
/// [`Manager`]: super::Manager
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ManagerConfig {
    /// Method of how a connection is recycled. See [`RecyclingMethod`].
    pub recycling_method: RecyclingMethod,
}
