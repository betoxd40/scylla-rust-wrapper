use deadpool_scylla::{Config, Manager, ManagerConfig, Pool};
use std::time::Duration;

async fn setup_pool() -> Pool {
    let config = Config::new(vec!["127.0.0.1".into()], 9042, "mykeyspace".into())
        .with_connection_timeout(Duration::from_secs(5));

    let manager_config = ManagerConfig::default();
    let manager = Manager::new(config, manager_config);

    Pool::builder(manager)
        .max_size(5)
        .build()
        .expect("Failed to create pool.")
}

#[tokio::test]
async fn test_connection_manager() {
    let pool = setup_pool().await;

    let conn_result = pool.get().await;
    assert!(
        conn_result.is_ok(),
        "Failed to get a connection from the pool."
    );
}

#[tokio::test]
async fn test_execute_query() {
    let pool = setup_pool().await;
    let conn = pool.get().await.expect("Failed to get a connection from the pool.");

    // Assuming conn is a valid session or similar that can execute queries
    // You'd need to adjust this part based on how you've set up query execution in your application
    let query = "SELECT now() FROM system.local";
    let execute_result = conn.execute(query).await;

    assert!(
        execute_result.is_ok(),
        "Failed to execute query on the pool connection."
    );
}
