use utoipa::openapi::{Info, OpenApi, OpenApiBuilder};
use crate::module::ModuleEntry;

pub fn merge_modules(modules: &[ModuleEntry], info: Info) -> OpenApi {
    let base = OpenApiBuilder::new().info(info).build();
    modules.iter().fold(base, |mut acc, e| {
        acc.merge(e.module.openapi());
        acc
    })
}
