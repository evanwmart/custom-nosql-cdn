use std::sync::Arc;
use warp::{Filter, Rejection, Reply, query};
use serde::Deserialize;
use crate::{database::Database, logging::SharedLogger};

#[derive(Deserialize)]
struct LogQuery {
    format: Option<String>,
}

pub async fn start_server(db: Arc<Database>, logger: Arc<SharedLogger>) {
    let db_filter = warp::any().map(move || Arc::clone(&db));
    let logger_filter = warp::any().map(move || Arc::clone(&logger));

    let log_route = warp::path("logs")
        .and(query::<LogQuery>())
        .and(logger_filter)
        .map(|query: LogQuery, logger: Arc<SharedLogger>| -> Box<dyn Reply> {
            let logs = logger.get_logs(query.format.as_deref() == Some("json"));

            match query.format.as_deref() {
                Some("json") => Box::new(warp::reply::json(&logs)),
                _ => Box::new(warp::reply::with_header(
                    format!(
                        "{}\n",
                        logs.iter().map(|log| log.message.to_string()).collect::<Vec<_>>().join("\n")
                    ),
                    "Content-Type",
                    "text/plain",
                )),                
            }
        });

    let get_route = warp::path!("get" / String)
        .and(db_filter.clone())
        .and_then(|key, db: Arc<Database>| async move {
            handle_get(key, db).await
        });

    let insert_route = warp::path!("insert" / String / String)
        .and(db_filter)
        .and_then(|key, value, db: Arc<Database>| async move {
            handle_insert(key, value, db).await
        });

    let routes = log_route.or(get_route).or(insert_route);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_get(key: String, db: Arc<Database>) -> Result<impl Reply, Rejection> {
    match db.get(&key) {
        Ok(Some(value)) => Ok(format!("Value: {}\n", String::from_utf8_lossy(&value))),
        Ok(None) => Ok("Key not found\n".to_string()),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn handle_insert(
    key: String,
    value: String,
    db: Arc<Database>,
) -> Result<impl Reply, Rejection> {
    match db.insert(&key, value.as_bytes()) {
        Ok(_) => Ok("Inserted successfully\n".to_string()),
        Err(_) => Err(warp::reject::not_found()),
    }
}
