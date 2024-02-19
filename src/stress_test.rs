use deadpool_scylla::{Config, Manager, Pool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

async fn create_test_table(pool: Arc<Pool>) {
    let client_wrapper = pool
        .get()
        .await
        .expect("Failed to get client from pool for table creation");
    let create_table_query = "
        CREATE TABLE IF NOT EXISTS mykeyspace.your_table (
            id UUID PRIMARY KEY,
            some_value TEXT
        )";
    client_wrapper
        .query(create_table_query, ())
        .await
        .expect("Failed to create test table");
}

async fn delete_test_table(pool: Arc<Pool>) {
    let client_wrapper = pool
        .get()
        .await
        .expect("Failed to get client from pool for table deletion");
    let delete_table_query = "DROP TABLE IF EXISTS mykeyspace.your_table";
    client_wrapper
        .query(delete_table_query, ())
        .await
        .expect("Failed to delete test table");
}

async fn setup_pool() -> Pool {
    let config = Config::new(
        vec!["127.0.0.1".into()],
        9042,
        "mykeyspace".into(),
        String::from("username"),
        String::from("password"),
    )
    .with_connection_timeout(Duration::from_secs(10));

    let manager = Manager::new(config);
    Pool::builder(manager).max_size(20).build().unwrap()
}

async fn perform_db_operation(pool: Arc<Pool>, semaphore: Arc<Semaphore>) {
    let _permit = semaphore
        .acquire()
        .await
        .expect("Failed to acquire semaphore permit");

    let client_wrapper = pool.get().await.expect("Failed to get client from pool");
    let query = "INSERT INTO your_table (id, some_value) VALUES (?, ?)";
    let id = Uuid::new_v4();
    let some_value = "test value";

    client_wrapper
        .query(query, (id, some_value))
        .await
        .expect("Failed to execute query");
}
#[tokio::main]
async fn main() {
    let pool = Arc::new(setup_pool().await);

    // Create the test table
    create_test_table(pool.clone()).await;

    let semaphore = Arc::new(Semaphore::new(100)); // Limit concurrency to 100

    let tasks: Vec<_> = (0..100000) // Simulate 100000 operations
        .map(|_| {
            let pool_clone = pool.clone();
            let semaphore_clone = semaphore.clone();
            tokio::spawn(async move {
                perform_db_operation(pool_clone, semaphore_clone).await;
            })
        })
        .collect();

    futures::future::join_all(tasks).await;

    // Delete the test table
    delete_test_table(pool.clone()).await;

    println!("Stress test completed.");
}
