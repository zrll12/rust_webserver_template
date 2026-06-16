use axum::Router;
use axum::routing::get;
use serde::Serialize;
use shadow_rs::shadow;
use ws_core::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ping))
}

async fn ping() -> String {
    shadow!(build);

    let pong = Pong {
        commit: build::SHORT_COMMIT.to_string(),
        branch: build::BRANCH.to_string(),
        date: build::COMMIT_DATE.to_string(),
        rust_version: build::RUST_VERSION.to_string(),
        build_channel: build::BUILD_RUST_CHANNEL.to_string(),
    };

    serde_json::to_string(&pong).unwrap()
}

#[derive(Serialize)]
struct Pong {
    commit: String,
    branch: String,
    date: String,
    rust_version: String,
    build_channel: String,
}
