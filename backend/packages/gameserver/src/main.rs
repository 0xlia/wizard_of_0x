use gameserver::{AppState, build_app};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gameserver=debug,tower_http=info".into()),
        )
        .init();

    let frontend_dir =
        std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "../frontend".to_owned());

    let app = build_app(AppState::new()).fallback_service(ServeDir::new(&frontend_dir));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    tracing::info!("serving frontend from {frontend_dir}");
    axum::serve(listener, app).await.unwrap();
}
