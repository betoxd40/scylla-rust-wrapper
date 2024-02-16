use deadpool_scylla::{Config, Manager, Pool};
use futures::future::join_all;
use scylla::{
    transport::downgrading_consistency_retry_policy::DowngradingConsistencyRetryPolicy,
    ExecutionProfile, IntoTypedRows, Session, SessionBuilder,
};
use std::time::Duration;
use uuid::Uuid;

async fn create_session_with_downgrading_policy() -> Result<Session, Box<dyn std::error::Error>> {
    let handle = ExecutionProfile::builder()
        .retry_policy(Box::new(DowngradingConsistencyRetryPolicy::new()))
        .build()
        .into_handle();

    let session: Session = SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .default_execution_profile_handle(handle)
        .build()
        .await?;

    Ok(session)
}

async fn setup_pool(max_size: usize, connection_timeout: Duration) -> Pool {
    let config: Config = Config::new(vec!["127.0.0.1".into()], 9042, "mykeyspace".into())
        .with_connection_timeout(connection_timeout);

    let manager = Manager::new(config);

    Pool::builder(manager)
        .max_size(max_size)
        .build()
        .expect("Failed to create pool.")
}

async fn setup_pool_for_timeout() -> Pool {
    let config = Config::new(vec!["10.255.255.1".into()], 9042, "mykeyspace".into())
        .with_connection_timeout(Duration::from_secs(1)); // Adjust timeout as needed

    Pool::builder(Manager::new(config))
        .max_size(1)
        .build()
        .expect("Failed to create pool.")
}

#[tokio::test]
async fn test_successful_connection() {
    let pool = setup_pool(5, Duration::from_secs(5)).await;
    let conn_result = pool.get().await;
    assert!(
        conn_result.is_ok(),
        "Failed to get a connection from the pool."
    );
}

const DROP_TABLE_QUERY: &str = "DROP TABLE IF EXISTS test_table";

async fn setup_test_environment(pool: &Pool) {
    let create_table_query = "
    CREATE TABLE IF NOT EXISTS mykeyspace.test_table (
        id UUID PRIMARY KEY,
        name TEXT,
        value INT
    )";

    let session = pool
        .get()
        .await
        .expect("Failed to get session from the pool for setup");
    session
        .query(create_table_query, ())
        .await
        .expect("Failed to create test table");
}

async fn teardown_test_environment(pool: &Pool) {
    let session = pool
        .get()
        .await
        .expect("Failed to get session from the pool for teardown");
    session
        .query(DROP_TABLE_QUERY, ())
        .await
        .expect("Failed to drop test table");
}

#[tokio::test]
async fn test_concurrent_connections() {
    let pool = setup_pool(5, Duration::from_secs(5)).await;
    let futures = (0..5).map(|_| {
        let pool_clone = pool.clone();
        async move {
            pool_clone
                .get()
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
            // This returns a Result<(), String>, satisfying the use of is_ok() later.
        }
    });

    // Use join_all to wait for all futures to complete
    let results = join_all(futures).await; // This will be a Vec<Result<(), String>>

    // Now you can iterate over the results and check if they are ok
    for result in results {
        assert!(result.is_ok(), "One or more concurrent connections failed.");
    }
}

#[tokio::test]
async fn test_connection_timeout() {
    let pool = setup_pool_for_timeout().await;
    let result = pool.get().await;
    assert!(
        result.is_err(),
        "Expected a timeout or connection error, but operation succeeded"
    );

    // Adjust the error assertion to be more inclusive of different timeout-related messages
    if let Err(e) = &result {
        let error_message = e.to_string();
        // Check if the error message contains any indication of a timeout
        assert!(
            error_message.contains("Timeout Error") || error_message.contains("timeout"),
            "Expected a timeout error, got a different error: {:?}",
            e
        );
    }
}

#[tokio::test]
async fn test_insert_data() {
    let pool = setup_pool(5, Duration::from_secs(5)).await;
    setup_test_environment(&pool).await;

    const INSERT_QUERY: &str =
        "INSERT INTO mykeyspace.test_table (id, name, value) VALUES (?, ?, ?)";
    let id = Uuid::new_v4();
    let name = "test";
    let value = 100;

    let session_with_retry_policy = create_session_with_downgrading_policy().await.unwrap();

    let result = session_with_retry_policy
        .query(INSERT_QUERY, (id, name, value))
        .await;

    assert!(result.is_ok(), "Failed to insert data into the table");

    teardown_test_environment(&pool).await;
}

#[tokio::test]
async fn test_update_data() {
    let pool = setup_pool(5, Duration::from_secs(5)).await;
    setup_test_environment(&pool).await;

    let session_with_retry_policy = create_session_with_downgrading_policy().await.unwrap();

    const INSERT_QUERY: &str =
        "INSERT INTO mykeyspace.test_table (id, name, value) VALUES (?, ?, ?)";
    let id = Uuid::new_v4();
    session_with_retry_policy
        .query(INSERT_QUERY, (id, "initial_name", 100))
        .await
        .expect("Failed to insert data for update test");

    const UPDATE_QUERY: &str = "UPDATE mykeyspace.test_table SET name = ?, value = ? WHERE id = ?";
    session_with_retry_policy
        .query(UPDATE_QUERY, ("updated_name", 200, id))
        .await
        .expect("Failed to update data");

    const SELECT_QUERY: &str = "SELECT name, value FROM mykeyspace.test_table WHERE id = ?";
    let rows = session_with_retry_policy
        .query(SELECT_QUERY, (id,))
        .await
        .expect("Failed to select data")
        .rows
        .expect("No rows found after update")
        .into_typed::<(String, i32)>()
        .next()
        .expect("No data found with the specified id")
        .expect("Failed to parse row data");

    assert_eq!(rows.0, "updated_name");
    assert_eq!(rows.1, 200);

    teardown_test_environment(&pool).await;
}

#[tokio::test]
async fn test_select_data() {
    let pool = setup_pool(5, Duration::from_secs(5)).await;
    setup_test_environment(&pool).await;

    let session_with_retry_policy = create_session_with_downgrading_policy().await.unwrap();

    const INSERT_QUERY: &str =
        "INSERT INTO mykeyspace.test_table (id, name, value) VALUES (?, ?, ?)";
    let id = Uuid::new_v4();
    session_with_retry_policy
        .query(INSERT_QUERY, (id, "test_select", 123))
        .await
        .expect("Failed to insert data for select test");

    const SELECT_QUERY: &str = "SELECT name, value FROM mykeyspace.test_table WHERE id = ?";
    let rows = session_with_retry_policy
        .query(SELECT_QUERY, (id,))
        .await
        .expect("Failed to select data")
        .rows
        .expect("No rows found")
        .into_typed::<(String, i32)>()
        .next()
        .expect("No data found with the specified id")
        .expect("Failed to parse row data");

    assert_eq!(rows.0, "test_select");
    assert_eq!(rows.1, 123);

    teardown_test_environment(&pool).await;
}

#[tokio::test]
async fn test_delete_data() {
    let pool = setup_pool(5, Duration::from_secs(5)).await;
    setup_test_environment(&pool).await;

    let session_with_retry_policy = create_session_with_downgrading_policy().await.unwrap();

    const INSERT_QUERY: &str =
        "INSERT INTO mykeyspace.test_table (id, name, value) VALUES (?, ?, ?)";
    let id = Uuid::new_v4();
    session_with_retry_policy
        .query(INSERT_QUERY, (id, "test_delete", 456))
        .await
        .expect("Failed to insert data for delete test");

    const DELETE_QUERY: &str = "DELETE FROM mykeyspace.test_table WHERE id = ?";
    session_with_retry_policy
        .query(DELETE_QUERY, (id,))
        .await
        .expect("Failed to delete data");

    const SELECT_QUERY: &str = "SELECT id FROM mykeyspace.test_table WHERE id = ?";
    let rows = session_with_retry_policy
        .query(SELECT_QUERY, (id,))
        .await
        .expect("Failed to select data after delete")
        .rows
        .expect("Error fetching rows after delete");

    assert!(
        rows.is_empty(),
        "Data found for id that should have been deleted"
    );

    teardown_test_environment(&pool).await;
}
