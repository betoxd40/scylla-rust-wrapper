# scylla_deadpool

s a Rust crate designed to simplify the process of managing ScyllaDB connections in an asynchronous Rust application. By leveraging deadpool-scylla, it offers a robust and efficient way to pool ScyllaDB connections for high-performance database operations. This crate abstracts the complexity of connection pooling and retries with downgrading consistency, making it easier for developers to focus on their application logic.

## Features

- Async/Await Support: Fully compatible with Rust's async/await syntax for non-blocking database operations.
- Connection Pooling: Efficiently manages a pool of connections to ScyllaDB, optimizing resource usage and throughput.
- Downgrading Consistency Retry Policy: Automatically retries queries with a downgradable consistency level, enhancing fault tolerance and availability.
- Easy Integration: Designed to be easily integrated into any async Rust application that requires access to ScyllaDB.

## Quick Start

Add `scylla-deadpool` to your `Cargo.toml`:

```toml
[dependencies]
scylla-deadpool = "0.1"
```

Create a connection pool and execute a query:

```rust
use your_crate_name::{Config, Manager, Pool};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Set up the pool configuration
    let config = Config::new(vec!["127.0.0.1:9042".into()], "mykeyspace".into())
        .with_connection_timeout(Duration::from_secs(5));

    // Initialize the connection pool
    let pool = Pool::new(Manager::new(config), 16);

    // Perform a database operation
    let client = pool.get().await.expect("Failed to get a client from the pool");
    let result = client.query("SELECT * FROM my_table", &[]).await;

    match result {
        Ok(rows) => println!("Query success: {:?}", rows),
        Err(e) => eprintln!("Query failed: {:?}", e),
    }
}
```
