use axum::Router;
use ws_core::module::AppModule;
use ws_core::state::AppState;

pub mod routes;

pub struct PingModule;

impl AppModule for PingModule {
    fn name(&self) -> &'static str {
        "ping"
    }

    fn routes(&self) -> Router<AppState> {
        routes::router()
    }
}
