mod database;
mod http;
mod logging;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize logging
    logging::init();

    // Create the database instance
    let db = Arc::new(database::Database::new("data.db".to_string()));

    // Start the HTTP server
    http::start_server(db).await;
}
