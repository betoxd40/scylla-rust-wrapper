// scylla_client.rs
use super::config::Config;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use tokio::sync::Mutex; // Adjust the path based on your project structure

/// A simple wrapper around the ScyllaDB `Session` to facilitate easy usage
/// within your application.
pub struct ScyllaClient {
    /// The `Session` object is wrapped in an `Arc<Mutex<>>` to allow for safe
    /// shared mutability across async tasks.
    session: Arc<Mutex<Session>>,
}

impl ScyllaClient {
    /// Creates and initializes a new `ScyllaClient` instance based on the provided configuration.
    pub async fn new(config: &Config) -> Result<Self, scylla::transport::errors::NewSessionError> {
        let session = SessionBuilder::new()
            .known_nodes(
                config
                    .hosts
                    .iter()
                    .map(|host| format!("{}:{}", host, config.port))
                    .collect::<Vec<_>>(),
            )
            .build()
            .await?;
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }
}
