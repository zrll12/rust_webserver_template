use axum::Router;
use thalos_core::{error::AppError, module::AppModule, state::AppState};

pub mod entity;
pub mod error;
pub mod extract;
pub mod routes;
pub mod service;

#[allow(unused_imports)]
pub use service::{TokenInfo, UserService};

pub struct UserModule;

impl AppModule for UserModule {
    fn name(&self) -> &'static str {
        "user"
    }

    fn routes(&self) -> Router<AppState> {
        routes::router()
    }

    fn init(&self, state: &AppState) -> Result<(), AppError> {
        futures::executor::block_on(
            state
                .db
                .get_schema_builder()
                .register(entity::user::Entity)
                .sync(&state.db),
        )
        .expect("user schema sync failed");
        state.set_module(UserService::new(state.db.clone()));
        Ok(())
    }

    #[cfg(feature = "openapi")]
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        use utoipa::OpenApi;

        #[derive(OpenApi)]
        #[openapi(
            paths(
                routes::register,
                routes::verify,
                routes::me,
            ),
            components(schemas(
                routes::RegisterRequest,
                routes::RegisterResponse,
                routes::VerifyRequest,
                routes::VerifyResponse,
                routes::MeResponse,
            )),
            security(("bearer_token" = [])),
        )]
        struct UserDoc;

        UserDoc::openapi()
    }
}
