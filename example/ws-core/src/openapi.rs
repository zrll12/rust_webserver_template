use utoipa::openapi::{Info, OpenApi, OpenApiBuilder};
use crate::module::AppModule;

/// 将所有模块的 OpenAPI 文档聚合为单个 OpenApi 对象。
///
/// 调用时机：启动时调用一次，结果缓存在 `OnceLock<String>` 中供运行时复用。
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
