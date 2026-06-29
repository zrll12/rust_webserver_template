use utoipa::openapi::{Info, OpenApi, OpenApiBuilder};
use crate::module::ModuleEntry;

/// Merges OpenAPI specs from all modules into a single `OpenApi` object.
pub fn merge_modules(
    modules: &[ModuleEntry],
    info: Info,
) -> OpenApi {
    let base = OpenApiBuilder::new().info(info).build();
    modules.iter().fold(base, |mut acc, e| {
        acc.merge(e.module.openapi());
        acc
    })
}
