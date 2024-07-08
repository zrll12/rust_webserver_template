use axum::{Router};
use axum::routing::get;

pub fn all_routers() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
}