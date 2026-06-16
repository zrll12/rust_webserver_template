/// 离线导出 OpenAPI spec，不需要运行服务器和数据库。
///
/// 运行方式：
///   cargo test --features openapi export_openapi -- --nocapture
///
/// 默认输出到项目根目录的 openapi.json。
/// 可通过环境变量 OPENAPI_OUTPUT 覆盖输出路径：
///   OPENAPI_OUTPUT=./docs/api.json cargo test --features openapi export_openapi
#[cfg(all(test, feature = "openapi"))]
mod tests {
    use utoipa::openapi::InfoBuilder;

    #[test]
    fn export_openapi() {
        let modules = crate::modules::all_modules();
        let info = InfoBuilder::new()
            .title(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .build();
        let openapi = ws_core::openapi::merge_modules(&modules, info);
        let json = openapi.to_json().expect("OpenAPI serialization failed");

        let output_path = std::env::var("OPENAPI_OUTPUT")
            .unwrap_or_else(|_| "openapi.json".to_string());
        std::fs::write(&output_path, &json)
            .unwrap_or_else(|e| panic!("Failed to write {output_path}: {e}"));

        println!("OpenAPI spec written to {output_path} ({} bytes)", json.len());
    }
}
