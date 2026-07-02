/// Exports OpenAPI spec offline without starting the server or database.
/// Usage: `cargo test --features openapi export_openapi -- --nocapture`
/// Override output path with `OPENAPI_OUTPUT` env var (default: openapi.json).
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
        let openapi = crate::openapi::merge_modules(&modules, info);
        let json = openapi.to_json().expect("OpenAPI serialization failed");

        let output_path = std::env::var("OPENAPI_OUTPUT")
            .unwrap_or_else(|_| "openapi.json".to_string());
        std::fs::write(&output_path, &json)
            .unwrap_or_else(|e| panic!("Failed to write {output_path}: {e}"));

        println!("OpenAPI spec written to {output_path} ({} bytes)", json.len());
    }
}
