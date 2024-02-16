use deadpool_scylla::{Config, Manager, ManagerConfig, Pool};
use futures::future::join_all;
use log::*;
use std::time::Duration;

async fn setup_pool(max_size: usize, connection_timeout: Duration) -> Pool {
    let config = Config::new(vec!["127.0.0.1".into()], 9042, "mykeyspace".into())
        .with_connection_timeout(connection_timeout);

    let manager_config = ManagerConfig::default();
    let manager = Manager::new(config, manager_config);

    Pool::builder(manager)
        .max_size(max_size)
        .build()
        .expect("Failed to create pool.")
}

#[tokio::test]
async fn test_max_connection_handling() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let max_size = 5;
    let pool = setup_pool(max_size, Duration::from_secs(5)).await;

    // Simulate a burst of connection requests greater than the pool's max size
    let total_requests = max_size * 2; // Request twice the number of connections in the pool
    let work_duration = Duration::from_secs(1); // Simulate longer work duration

    let futures: Vec<_> = (0..total_requests)
        .map(|_| {
            let pool_clone = pool.clone();
            async move {
                match pool_clone.get().await {
                    Ok(conn) => {
                        // Simulate work that holds onto the connection longer than the previous test
                        tokio::time::sleep(work_duration).await;
                        drop(conn); // Explicitly drop the connection to return it to the pool
                        true
                    }
                    Err(e) => {
                        warn!("Failed to acquire connection from pool: {:?}", e);
                        false
                    }
                }
            }
        })
        .collect();

    // Use join_all to wait for all futures to complete
    let results = join_all(futures).await;

    // Analyze the results to determine how many connection requests were successful
    let successful_requests = results.into_iter().filter(|&succeeded| succeeded).count();

    // Assert that we were able to successfully acquire and release connections
    // The exact behavior here depends on how you expect the pool to handle over-saturation
    assert!(
        successful_requests > 0,
        "Expected to successfully handle at least some connections."
    );
}

#[tokio::test]
async fn test_connection_distribution_and_recycling() {
    let max_size = 5;
    let pool = setup_pool(max_size, Duration::from_secs(5)).await;

    // Simulate varied workload
    let work_times = vec![100, 200, 300, 400, 500]; // Milliseconds

    let futures: Vec<_> = work_times
        .into_iter()
        .map(|work_time| {
            let pool_clone = pool.clone();
            async move {
                let conn = pool_clone
                    .get()
                    .await
                    .expect("Failed to acquire connection");
                // Simulate variable work time
                tokio::time::sleep(Duration::from_millis(work_time)).await;
                // Implicitly release connection by dropping `conn` at the end of the scope
            }
        })
        .collect();

    // Wait for all tasks to complete
    join_all(futures).await;

    // Since connections are released back to the pool, we test if we can acquire max connections again
    let test_futures: Vec<_> = (0..max_size)
        .map(|_| {
            let pool_clone = pool.clone();
            async move {
                pool_clone
                    .get()
                    .await
                    .expect("Failed to acquire connection after recycling");
            }
        })
        .collect();

    join_all(test_futures).await;

    // If we reach this point without panic or error, it means connections were correctly distributed and recycled
}
