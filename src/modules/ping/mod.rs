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

    #[cfg(feature = "openapi")]
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        use utoipa::OpenApi;

        #[derive(OpenApi)]
        #[openapi(paths(routes::ping), components(schemas(routes::Pong)))]
        struct PingDoc;

        PingDoc::openapi()
    }
}
