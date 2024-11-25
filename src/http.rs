use std::sync::Arc;
use warp::Filter;
use warp::reject::Reject;
use crate::database::Database;

#[derive(Debug)]
struct DatabaseError;

impl Reject for DatabaseError {}

pub async fn start_server(db: Arc<Database>) {
    let db_filter = warp::any().map(move || Arc::clone(&db));

    let log_route = warp::path("logs").map(|| warp::reply::html("Log service placeholder"));

    let get_route = warp::path!("get" / String)
        .and(db_filter.clone())
        .and_then(handle_get);

    let insert_route = warp::path!("insert" / String / String)
        .and(db_filter)
        .and_then(handle_insert);

    let routes = log_route.or(get_route).or(insert_route);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_get(key: String, db: Arc<Database>) -> Result<impl warp::Reply, warp::Rejection> {
    match db.get(&key) {
        Ok(Some(value)) => Ok(format!("Value: {}", String::from_utf8_lossy(&value))),
        Ok(None) => Ok("Key not found".to_string()),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}

async fn handle_insert(key: String, value: String, db: Arc<Database>) -> Result<impl warp::Reply, warp::Rejection> {
    match db.insert(&key, value.as_bytes()) {
        Ok(_) => Ok("Inserted successfully".to_string()),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}
