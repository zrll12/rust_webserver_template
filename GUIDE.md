# 使用指南

基于 Axum + SeaORM 的模块化 Web 服务脚手架。`cargo generate` 后只需维护 `src/modules/` 下的业务模块，框架层随 template 更新。

## 快速开始

```bash
cargo generate --git https://github.com/zrll12/rust_webserver_template.git
cd your_project
cargo run
```

首次运行会在 `config/` 目录自动生成配置文件，修改 `config/core.toml` 填写数据库连接后重启即可。

---

## 目录结构

```
src/
├── main.rs                  # 启动入口，不需要修改
└── modules/
    ├── mod.rs               # ← 注册模块的唯一入口，你只需改这里
    └── ping/                # 内置示例模块
ws-core/                     # 框架层（独立 crate），不需要修改
config/
    core.toml                # 服务器、数据库、TLS、CORS 配置
```

**原则：框架层不知道业务层的存在。** 框架只通过 `AppModule` trait 调用业务代码，不 import 任何业务类型。

---

## 添加一个模块

### 1. 创建模块目录

```
src/modules/your_module/
├── mod.rs        必需：定义 YourModule struct，impl AppModule
├── routes.rs     必需：路由函数，返回 Router<AppState>
├── entity/       可选：sea-orm entity 定义
├── service.rs    可选：业务逻辑，需 derive Clone
├── error.rs      可选：模块私有错误
└── config.rs     可选：模块私有配置
```

### 2. 实现 AppModule

**最简形式（无服务注入）：**

```rust
// src/modules/your_module/mod.rs
use axum::Router;
use ws_core::module::AppModule;
use ws_core::state::AppState;

pub mod routes;

pub struct YourModule;

impl AppModule for YourModule {
    fn name(&self) -> &'static str { "your_module" }
    fn routes(&self) -> Router<AppState> { routes::router() }
}
```

**需要向其他模块暴露服务时，实现 `init`：**

```rust
use ws_core::error::AppError;

impl AppModule for YourModule {
    fn name(&self) -> &'static str { "your_module" }
    fn routes(&self) -> Router<AppState> { routes::router() }

    fn init(&self, state: &AppState) -> Result<(), AppError> {
        state.set_module(YourService::new(state.db.clone()));
        Ok(())
    }
}
```

`init` 是同步的。需要异步操作时用 `futures::executor::block_on(...)` 包裹。

**在路由 handler 中消费服务：**

```rust
use ws_core::extract::ModuleExt;

async fn handler(
    ModuleExt(svc): ModuleExt<YourService>,
    Json(body): Json<Request>,
) -> Result<Json<Response>, AppError> {
    // svc 是 Arc<YourService>
    Ok(Json(svc.do_something(body).await?))
}
```

### 3. 注册模块

```rust
// src/modules/mod.rs
pub mod your_module;

pub fn all_modules() -> Vec<Box<dyn AppModule>> {
    vec![
        Box::new(ping::PingModule),
        Box::new(your_module::YourModule),
        // 顺序即初始化顺序：被依赖的模块写在前面
    ]
}
```

---

## 数据库（Entity First）

不需要手写 migration，直接定义 entity，启动时自动同步表结构：

```rust
// src/modules/your_module/entity/item.rs
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub created_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
```

启动时自动对比 schema，增量执行 DDL（只加不删）：

| 变化 | 自动执行 |
|---|---|
| 新增 entity | `CREATE TABLE` |
| 新增字段 | `ALTER TABLE ADD COLUMN` |
| 重命名字段（加 `#[sea_orm(renamed_from = "...")]`） | `ALTER TABLE RENAME COLUMN` |
| 新增/移除 `#[sea_orm(unique)]` | `CREATE/DROP INDEX` |
| 删除字段或表 | **不执行** |

---

## 模块间通信

模块不能直接调用对方的函数，通过 `AppState` 共享服务：

```rust
// 提供方：init 中注册
state.set_module(AuthService::new(state.db.clone()));

// 消费方：handler 用 ModuleExt<T> extractor
async fn handler(
    ModuleExt(auth): ModuleExt<AuthService>,
    ...
) -> Result<..., AppError> { ... }
```

在 `all_modules()` 中确保提供方排在消费方前面。

---

## 错误处理

**启动阶段**（`init`、配置加载）：直接 `panic` / `expect`，让进程尽早退出。

**运行阶段**（路由 handler）：返回 `Result<T, AppError>`，用 `?` 传播，不 `unwrap`。

模块私有错误通过 `From` 转换到 `AppError`：

```rust
impl From<YourError> for AppError {
    fn from(e: YourError) -> Self {
        match e {
            YourError::NotFound => AppError::NotFound,
            YourError::Invalid(msg) => AppError::InvalidField {
                field: "field_name".into(),
                reason: msg,
            },
        }
    }
}
```

`AppError` 的变体：

| 变体 | HTTP 状态码 |
|---|---|
| `InvalidToken` | 401 |
| `PermissionDenied` | 403 |
| `NotFound` | 404 |
| `TooManySubmit` | 429 |
| `InvalidField { field, reason }` | 400 |
| `DatabaseError(DbErr)` | 500 |

---

## 配置

框架配置在 `config/core.toml`，首次启动自动生成。

模块可以有独立配置文件，首次启动自动生成默认值：

```rust
// src/modules/your_module/config.rs
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use ws_core::config::get_config;

#[serde_inline_default]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct YourConfig {
    #[serde_inline_default(8080u16)]
    pub port: u16,
}

impl YourConfig {
    pub fn load() -> Self { get_config("your_module") }
}
```

配置文件路径为 `config/your_module.toml`，修改配置后重启生效。更新结构体添加新字段时，下次启动自动写入默认值，已有注释保留。

---

## OpenAPI（可选）

启用 `openapi` feature 后，模块可以 override `openapi()` 方法提供文档：

```rust
#[cfg(feature = "openapi")]
fn openapi(&self) -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(routes::your_handler), components(schemas(YourSchema)))]
    struct Doc;

    Doc::openapi()
}
```

框架启动时聚合所有模块的文档，通过 `/openapi.json` 暴露。
