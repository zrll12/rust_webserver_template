## Rust Web Backend Template

```bash
cargo generate --git https://github.com/zrll12/thalos.git
```

Axum + SeaORM 模块化 Web 服务脚手架。生成后只需维护 `src/modules/` 下的业务模块。

详见 [GUIDE.md](GUIDE.md)。

---

## OpenAPI

可选 feature，默认关闭，不开启时零额外开销。

### Features

| Feature | 作用 |
|---|---|
| `openapi` | 启动时聚合文档，挂载 `/openapi.json` |
| `swagger-ui` | 在 `openapi` 基础上，额外挂载 `/swagger-ui` |

### 运行时文档

```bash
# 仅 JSON 端点
cargo run --features openapi

# JSON + Swagger UI
cargo run --features swagger-ui
```

启动后访问：
- `http://localhost:7890/openapi.json`
- `http://localhost:7890/swagger-ui`（仅 `swagger-ui` feature）

### 离线导出（无需数据库）

```bash
cargo test --features openapi export_openapi -- --nocapture

# 自定义输出路径
OPENAPI_OUTPUT=./docs/api.json cargo test --features openapi export_openapi -- --nocapture
```

### 给模块添加文档

**1. 标注 handler 和 schema**（`routes.rs`）：

```rust
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Serialize)]
pub(super) struct FooResponse { /* ... */ }

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/foo",
    responses((status = 200, body = FooResponse))
))]
pub(super) async fn foo_handler() -> Json<FooResponse> { /* ... */ }
```

**2. 实现 `openapi()`**（`mod.rs`）：

```rust
#[cfg(feature = "openapi")]
fn openapi(&self) -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(routes::foo_handler), components(schemas(routes::FooResponse)))]
    struct FooDoc;

    FooDoc::openapi()
}
```

框架会在启动时自动聚合所有模块的文档。未实现 `openapi()` 的模块使用默认空实现，不影响其他模块。
