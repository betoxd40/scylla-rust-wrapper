#[cfg(test)]
mod tests {
    use deadpool_scylla::{Config, Manager, Pool};
    use openssl::ssl::{SslContextBuilder, SslFiletype, SslMethod, SslVerifyMode};

    use super::*;
    use std::{path::Path, time::Duration};

    #[tokio::test]
    async fn test_ssl_session_creation() {
        // Adjust these values based on your test environment
        let hosts = vec!["scylladb-with-failover.infra.quiknode.net".to_string()];
        let port = 9142; // This is often the default port, adjust if necessary
        let keyspace = "solace_ledger".to_string();
        let ca_cert_path = "/Users/beto/Documents/deadpool_scylla/ca.crt".to_string();

        let config = Config::new(hosts, port, keyspace)
            .with_auth_and_cert(
                "solace_ledger_scylla".to_string(),
                "ABirq*AGQxJ!R!F-G6-RrQmH".to_string(),
                ca_cert_path,
            );

        // THIS WORKS
        // let mut ssl_context_builder = SslContextBuilder::new(SslMethod::tls()).unwrap();
        // ssl_context_builder
        //     .set_certificate_file(Path::new(&ca_cert_path), SslFiletype::PEM)
        //     .unwrap();
        // ssl_context_builder.set_verify(SslVerifyMode::NONE); // Consider using VERIFY_PEER for production environments
        // let session = scylla::SessionBuilder::new()
        //     .known_nodes(&["scylladb-with-failover.infra.quiknode.net:9142"])
        //     .user(&user, &password)
        //     .ssl_context(Some(ssl_context_builder.build()))
        //     .build()
        //     .await
        //     .expect("Failed to connect to ScyllaDB");
        // Execute a simple query to test the connection
        // let result = session.query("SELECT now() FROM system.local", &[]).await;

        let manager = Manager::new(config);

        let pool = Pool::builder(manager).max_size(20).build();

        match pool {
            Ok(conn) => {
                let conn = conn.get().await.unwrap();
                // assert query works
                assert_eq!(
                    conn.query("SELECT now() FROM system.local", &[]).await.is_ok(),
                    true
                );
            }
            Err(e) => {
                panic!("Error creating pool: {:?}", e);
            }
        }
    }
}
