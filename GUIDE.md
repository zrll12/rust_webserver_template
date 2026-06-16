# rust_webserver_template 使用指南

基于 Axum + SeaORM 的模块化 Web 服务脚手架。每次新项目只需维护自己的业务模块，框架层随 template 更新。

## 快速开始

```bash
cargo generate --git https://github.com/zrll12/rust_webserver_template.git
```

生成后唯一需要修改的框架侧文件是 `src/modules/mod.rs`，在其中注册自己的模块。

---

## 项目结构

```
src/
├── main.rs                  # 启动入口，通常不需要修改
├── core/                    # 框架层，随 template 维护，通常不需要修改
│   ├── module.rs            # AppModule trait
│   ├── state.rs             # AppState（db + config）
│   ├── error.rs             # AppError
│   └── config/              # TOML 配置工具
└── modules/                 # 业务层，只维护这里
    ├── mod.rs               # ← 注册模块的唯一入口
    └── ping/                # 内置示例模块
config/
    core.toml                # 服务器、数据库、TLS、CORS 配置，首次启动自动生成
example/                     # 完整示例项目，展示 user 模块的写法
```

**原则：框架层不知道业务层的存在。** 框架只调用 `AppModule` trait，不 import 任何业务代码。

---

## 添加一个模块

### 1. 创建模块目录

```
src/modules/your_module/
├── mod.rs        必需
├── routes.rs     必需
├── entity/       可选，sea-orm entity
├── service.rs    可选，业务逻辑
├── error.rs      可选，模块私有错误
└── config.rs     可选，模块私有配置
```

### 2. 实现 AppModule

**不需要暴露服务的模块（最简形式）：**

```rust
// src/modules/your_module/mod.rs
use async_trait::async_trait;
use axum::Router;
use crate::core::{module::AppModule, state::AppState};

pub mod routes;

pub struct YourModule;

#[async_trait]
impl AppModule for YourModule {
    fn name(&self) -> &'static str { "your_module" }
    fn routes(&self) -> Router<AppState> { routes::router() }
}
```

**需要向外暴露服务的模块：**

```rust
// src/modules/your_module/mod.rs
use std::future::Future;
use std::pin::Pin;
use axum::Router;
use crate::core::{error::AppError, module::AppModule, state::AppState};

pub mod routes;
pub mod service;
pub use service::YourService;

pub struct YourModule;

impl AppModule for YourModule {
    fn name(&self) -> &'static str { "your_module" }
    fn routes(&self) -> Router<AppState> { routes::router() }

    fn init<'a>(&'a self, state: &'a AppState) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            state.set_module(YourService::new(state.db.clone()));
            Ok(())
        })
    }
}
```

### 3. 注册模块

在 `src/modules/mod.rs` 中添加：

```rust
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

## Entity First（数据库表结构）

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

启动时 sea-orm 自动对比数据库 schema，增量执行 DDL（只加不删）：

| 变化 | 自动执行 |
|---|---|
| 新增 entity | `CREATE TABLE` |
| 新增字段 | `ALTER TABLE ADD COLUMN` |
| 重命名字段（需加 `#[sea_orm(renamed_from = "...")]`） | `ALTER TABLE RENAME COLUMN` |
| 新增/移除 `#[sea_orm(unique)]` | `CREATE/DROP INDEX` |
| 删除字段或表 | 不执行 |

---

## 模块间通信

模块不能直接调用对方的内部函数，通过 `AppState` 的 module map 共享服务：

```rust
// 提供服务：在 init 里注册
state.set_module(AuthService::new(state.db.clone()));

// 消费服务：路由函数用 ModuleExt<T> extractor
use crate::core::extract::ModuleExt;

async fn handler(
    ModuleExt(auth): ModuleExt<AuthService>,  // 由 AuthModule::init 注册
    Json(body): Json<Request>,
) -> Result<Json<Response>, AppError> { ... }
```

在 `all_modules()` 中确保提供服务的模块排在前面。

---

## 错误处理

**启动阶段**（`init`、配置加载、数据库连接）：直接 `panic` / `expect`，让进程尽早退出。

**运行阶段**（路由 handler）：一律返回 `Result<T, AppError>`，用 `?` 传播，绝不 `unwrap`。

模块私有错误通过 `From` 转换到 `AppError`：

```rust
impl From<YourError> for AppError {
    fn from(e: YourError) -> Self {
        match e {
            YourError::NotFound => AppError::NotFound,
            YourError::Invalid(msg) => AppError::InvalidField {
                field: "...".into(),
                reason: msg,
            },
        }
    }
}
```

---

## 配置

框架配置在 `config/core.toml`，首次启动自动生成，包含服务地址、数据库连接、TLS、CORS 等。

模块可以有独立配置文件 `config/<module_name>.toml`，首次启动自动生成默认值：

```rust
// src/modules/your_module/config.rs
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use crate::core::config::get_config;

#[serde_inline_default]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct YourConfig {
    #[serde_inline_default(String::from("default_value"))]
    pub some_key: String,
}

impl YourConfig {
    pub fn load() -> Self { get_config("your_module") }
}
```

---

## 示例项目

`example/` 目录是一个完整的独立项目，展示了 `user` 模块的完整写法（entity、service、routes、error）。`cargo generate` 时不会包含此目录。
