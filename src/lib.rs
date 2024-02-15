#![deny(rust_2018_idioms, nonstandard_style)]
#![forbid(unsafe_code)]

mod config;
mod generic_client;

use async_trait::async_trait;
use deadpool::managed;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;

pub use self::config::{Config, ManagerConfig};
pub use scylla;


// Re-export deadpool's managed module for convenience
pub use deadpool::managed::reexports::*;

pub use self::generic_client::GenericClient;


// Define the Manager struct for ScyllaDB, adapting from the PostgreSQL example
pub struct Manager {
    config: ManagerConfig,
    scylla_config: Config, // This should be your custom config type, suitable for ScyllaDB
}

impl Manager {
    // Initialize a new Manager with ScyllaDB config
    pub fn new(scylla_config: Config, manager_config: ManagerConfig) -> Self {
        Self {
            config: manager_config,
            scylla_config,
        }
    }
}

pub type Pool = deadpool::managed::Pool<Manager, deadpool::managed::Object<Manager>>;


// Implement the deadpool Manager trait for your Manager
#[async_trait]
impl managed::Manager for Manager {
    type Type = ClientWrapper;
    type Error = Box<dyn std::error::Error>;

    // The create method now seamlessly integrates with async/await and the trait's expected lifetimes
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let session = SessionBuilder::new()
            .known_nodes(&self.scylla_config.hosts) // Ensure this matches your Config struct's fields
            .build()
            .await?;
        Ok(ClientWrapper::new(session))
    }

   async fn recycle(&self, client: &mut Self::Type, _metrics: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
    let query = "SELECT now() FROM system.local"; // Lightweight query to check connection

    let session_lock = client.session.lock().await; // Ensure you have async lock acquisition
    match session_lock.query(query, &[]).await {
        Ok(_) => Ok(()),
        Err(e) => {
            // Log or handle the error as needed
            Err(managed::RecycleError::Message(e.to_string()))
        }
    }
}
}

// Define your ClientWrapper and any additional functionality you need
pub struct ClientWrapper {
    session: Arc<Mutex<Session>>, // Use Arc<Mutex<>> to safely share and mutate across async tasks
}

impl ClientWrapper {
    pub fn new(session: Session) -> Self {
        Self {
            session: Arc::new(Mutex::new(session)),
        }
    }

    // Expose session methods or add custom behavior here
}
