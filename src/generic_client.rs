use async_trait::async_trait;
use cdrs_tokio::cluster::session::Session;
use cdrs_tokio::cluster::TcpConnectionManager;
use cdrs_tokio::error::Result as CdrsResult;
use cdrs_tokio::load_balancing::RoundRobinLoadBalancingStrategy;
use cdrs_tokio::transport::TransportTcp;
use std::sync::Arc;
use tokio::sync::Mutex;

mod private {
    pub trait Sealed {}
    impl Sealed for super::CdrsClientWrapper {}
}

type CdrsSession = Session<
    TransportTcp,
    TcpConnectionManager,
    RoundRobinLoadBalancingStrategy<TransportTcp, TcpConnectionManager>,
>;

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
