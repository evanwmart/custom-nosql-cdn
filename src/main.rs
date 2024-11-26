// src/main.rs

use custom_nosql_cdn::{database, http, logging};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize logging
    let logger = logging::init_logger();

    // Create the database instance
    let db = Arc::new(database::Database::new("data.db".to_string()));

    // Start the HTTP server
    http::start_server(db, logger).await;
}
