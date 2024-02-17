#![deny(rust_2018_idioms, nonstandard_style)]
#![forbid(unsafe_code)]

mod config;

use async_trait::async_trait;
use deadpool::managed;
use scylla::prepared_statement::PreparedStatement;
use scylla::query::Query;
use scylla::transport::downgrading_consistency_retry_policy::DowngradingConsistencyRetryPolicy;
use scylla::ExecutionProfile;
use scylla::{
    serialize::row::SerializeRow, transport::errors::QueryError, QueryResult, Session,
    SessionBuilder,
};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;

pub use self::config::Config;
pub use deadpool::managed::reexports::*;
pub use scylla;

pub struct Manager {
    config: Config,
}

impl Manager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

pub type Pool = deadpool::managed::Pool<Manager, deadpool::managed::Object<Manager>>;

#[async_trait]
impl managed::Manager for Manager {
    type Type = ClientWrapper;
    type Error = Box<dyn std::error::Error>;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let handle = ExecutionProfile::builder()
            .retry_policy(Box::new(DowngradingConsistencyRetryPolicy::new()))
            .build()
            .into_handle();

        let session = SessionBuilder::new()
            .known_nodes(&self.config.hosts)
            .use_keyspace(&self.config.keyspace, false)
            .default_execution_profile_handle(handle)
            .build()
            .await?;
        Ok(ClientWrapper::new(session))
    }

    async fn recycle(
        &self,
        client: &mut Self::Type,
        _metrics: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
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

pub struct ClientWrapper {
    session: Arc<Mutex<Session>>,
}

impl ClientWrapper {
    pub fn new(session: Session) -> Self {
        Self {
            session: Arc::new(Mutex::new(session)),
        }
    }

    pub async fn query(
        &self,
        statement: &str,
        values: impl SerializeRow + Send + Sync,
    ) -> Result<QueryResult, QueryError> {
        let session = self.session.lock().await;
        session.query(statement, values).await
    }

    pub async fn prepare(&self, statement: &str) -> Result<PreparedStatement, QueryError> {
        let session = self.session.lock().await;
        session.prepare(statement).await
    }

    pub async fn execute(&self, statement: &PreparedStatement) -> Result<(), QueryError> {
        let session = self.session.lock().await;
        // Execute the prepared statement without binding additional values
        session
            .execute(statement, &[])
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl Deref for ClientWrapper {
    type Target = Arc<Mutex<Session>>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl DerefMut for ClientWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}
