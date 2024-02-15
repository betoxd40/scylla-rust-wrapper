use async_trait::async_trait;
use cdrs_tokio::query::*;
use cdrs_tokio::types::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::future::Future;
use cdrs_tokio::error::Error;
use cdrs_tokio::error::Result as CdrsResult;
use cdrs_tokio::cluster::session::{Session, SessionBuilder, TcpSessionBuilder};
use cdrs_tokio::transport::TransportTcp;
use cdrs_tokio::cluster::{NodeTcpConfigBuilder, TcpConnectionManager};
use cdrs_tokio::load_balancing::RoundRobinLoadBalancingStrategy;

mod private {
    pub trait Sealed {}
    impl Sealed for super::CdrsClientWrapper {}
}


type CdrsSession = Session<TransportTcp, TcpConnectionManager, RoundRobinLoadBalancingStrategy<TransportTcp, TcpConnectionManager>>;

#[async_trait]
pub trait GenericClient: Sync + private::Sealed {
    async fn execute(&self, query: &str) -> CdrsResult<()>;
}

pub struct CdrsClientWrapper {
    // Assuming `session` is an instance of a CDRS Session compatible with the async query execution
    session: Arc<Mutex<CdrsSession>>,
}

#[async_trait]
impl GenericClient for CdrsClientWrapper {
    async fn execute(&self, query: &str) -> CdrsResult<()> {
        let session_guard = self.session.lock().await;
        session_guard.query(query).await?;
        Ok(())
    }
}