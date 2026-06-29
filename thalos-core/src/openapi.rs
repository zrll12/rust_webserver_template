use utoipa::openapi::{Info, OpenApi, OpenApiBuilder};
use crate::module::AppModule;

/// Merges OpenAPI specs from all modules into a single `OpenApi` object.
pub fn merge_modules(
    modules: &[Box<dyn AppModule>],
    info: Info,
) -> OpenApi {
    let base = OpenApiBuilder::new().info(info).build();
    modules.iter().fold(base, |mut acc, m| {
        acc.merge(m.openapi());
        acc
    })
}
