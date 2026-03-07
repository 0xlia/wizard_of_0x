use axum::{Router, routing::{get, post}, debug_handler};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateInput {

}

#[tokio::main]
async fn main() {
    let app = Router::new()
    .route("/", get(root_handler))
    .route("/create", post(create_game));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

#[debug_handler]
async fn root_handler() -> &'static str {
    "Hello World\n"
}

#[debug_handler]
async fn create_game() {

}