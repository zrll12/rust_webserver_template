# thalos — AI 协作指南

完整使用说明见 [GUIDE.md](GUIDE.md)。本文件供 AI 辅助开发时参考，记录约定、精确 API 签名和边界。

---

## 项目定位

Axum + SeaORM 的模块化 Web 服务脚手架（cargo-generate template）。用户 `cargo generate` 后只维护 `src/modules/` 下的业务模块，框架层（`thalos-core` crate、`src/main.rs`）随 crate 版本更新，用户不直接修改。

---

## 目录职责边界

| 路径 | 职责 | 用户是否修改 |
|---|---|---|
| `thalos-core` (crates.io) | 框架基础设施：AppModule trait、AppState、AppError、config 工具（独立 crate，不在本地目录） | 否 |
| `src/modules/mod.rs` | 模块注册表，`all_modules()` 返回所有模块 | **是，唯一必须改的框架侧文件** |
| `src/modules/<name>/` | 业务模块，每个子目录完全独立 | 是 |
| `src/main.rs` | 启动入口：logging、模块初始化、router 组装 | 否 |
| `config/` | TOML 配置文件，运行时读取，首次启动自动生成 | 是（运维配置） |

---

## 核心 API

### AppModule trait（`thalos-core/src/module.rs`）

```rust
pub trait AppModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;        // 模块名，同时是路由前缀
    fn routes(&self) -> Router<AppState>;  // 模块路由

    fn init(&self, _state: &AppState) -> Result<(), AppError> {
        Ok(())  // 默认空实现，同步，异步操作用 futures::executor::block_on 包裹
    }

    #[cfg(feature = "openapi")]
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        utoipa::openapi::OpenApiBuilder::new().build()  // 默认空文档
    }
}
```

### AppState（`thalos-core/src/state.rs`）

```rust
#[derive(Clone)]
pub struct AppState {
    pub core_config: CoreConfig,
    pub db: DatabaseConnection,   // sea-orm，内部是 Arc，clone 廉价
    // extensions: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>（私有）
}

impl AppState {
    pub fn set_module<T: Send + Sync + 'static>(&self, val: T);
    pub fn get_module<T: Send + Sync + 'static>(&self) -> Option<Arc<T>>;
}
```

### ModuleExt extractor（`thalos-core/src/extract.rs`）

```rust
pub struct ModuleExt<T>(pub Arc<T>);
// impl FromRequestParts<AppState>
// 从 AppState::get_module::<T>() 取值，取不到时返回 AppError::NotFound
```

### AppError（`thalos-core/src/error.rs`）

```rust
pub enum AppError {
    InvalidToken,                              // 401
    PermissionDenied,                          // 403
    NotFound,                                  // 404
    TooManySubmit,                             // 429
    InvalidField { field: String, reason: String }, // 400
    DatabaseError(#[from] sea_orm::DbErr),     // 500
}
```

### get_config（`thalos-core/src/config/mod.rs`）

```rust
pub fn get_config<T>(name: &str) -> T
where T: for<'a> Deserialize<'a> + Serialize
// 读取 config/{name}.toml，文件不存在时创建，缺字段时写入默认值，保留注释
```

---

## 模块结构约定

```
src/modules/<name>/
├── mod.rs       定义 <Name>Module struct，impl AppModule（必需）
├── routes.rs    路由函数，返回 Router<AppState>（必需）
├── entity/      sea-orm entity，带 #[sea_orm::model] 宏（可选）
├── service.rs   业务逻辑，必须 derive Clone（可选）
├── error.rs     模块私有错误，impl From<_> for AppError（可选）
└── config.rs    模块私有配置，用 get_config("<name>") 读取（可选）
```

正确的 import 路径（模板生成后 crate 名为项目名，thalos-core 以 `thalos_core` 引入）：

```rust
use thalos_core::module::AppModule;
use thalos_core::state::AppState;
use thalos_core::error::AppError;
use thalos_core::extract::ModuleExt;
use thalos_core::config::get_config;
```

### 服务注入模式

```rust
// init 中注册（mod.rs）
fn init(&self, state: &AppState) -> Result<(), AppError> {
    state.set_module(MyService::new(state.db.clone()));
    Ok(())
}

// handler 中消费（routes.rs）
async fn handler(ModuleExt(svc): ModuleExt<MyService>, ...) -> Result<..., AppError> {
    // svc 是 Arc<MyService>
}
```

### Schema Sync 模式

每个模块在 `init` 中负责同步自己的 entity，使用 `get_schema_builder()` 手动注册：

```rust
fn init(&self, state: &AppState) -> Result<(), AppError> {
    futures::executor::block_on(
        state
            .db
            .get_schema_builder()
            .register(entity::foo::Entity)
            .register(entity::bar::Entity)
            .sync(&state.db),
    )
    .expect("schema sync failed");
    Ok(())
}
```

- 只注册本模块自己的 entity，外部 crate 模块同样在自己的 `init` 里调用
- `#[sea_orm::model]` 宏仍然需要（用于派生 ORM 相关 trait），但不依赖全局 inventory 做 sync

### 模块注册顺序

`all_modules()` 的顺序即 `init()` 的执行顺序。模块 B 的 `init` 依赖模块 A 注册的服务时，A 必须排在 B 前面。

---

## 数据库（Entity First）

- sea-orm 2.x entity-first，**不存在 migration crate**
- entity 带 `#[sea_orm::model]` 宏（派生 ORM trait 所需）
- schema sync 在各模块的 `init` 中显式调用，增量执行 DDL，只增不删（index 除外）

---

## 错误处理规则

- **`init` 及启动阶段**：`expect`/`panic`，让进程尽早退出
- **路由 handler**：必须返回 `Result<T, AppError>`，用 `?` 传播，禁止 `unwrap`

---

## 禁止事项

- 不在 `thalos-core` crate 中 import 任何 `src/modules/` 的类型
- 不跨模块直接调用函数，通过 `state.set_module`/`ModuleExt<T>` 通信
- 不在 `modules/mod.rs` 之外注册模块
- 不在 entity 文件中放业务逻辑
- 不在路由 handler 中 `panic`/`unwrap`

---

## 依赖说明

| crate | 用途 |
|---|---|
| `axum 0.8` | HTTP 框架 |
| `sea-orm 2.0.0-rc.x` | ORM，features: `schema-sync`, `entity-registry` |
| `dashmap` | AppState 的 module extension map（线程安全 HashMap） |
| `tower-http` | middleware：trace、cors、catch-panic |
| `axum-server` | 支持 TLS 的服务器绑定 |
| `serde-inline-default` | 配置结构体字段默认值 |
| `shadow-rs` | 编译时注入 git/build 信息（ping 模块用） |
| `diff` | 配置文件 save_changes 时保留注释的 diff 合并 |
| `utoipa` | OpenAPI 文档生成（`openapi` feature） |
