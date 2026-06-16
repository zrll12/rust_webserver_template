# rust_webserver_template — AI 协作指南

完整使用说明见 [GUIDE.md](GUIDE.md)。本文件补充 AI 辅助开发时需要了解的约定和边界。

---

## 项目定位

这是一个 Axum + SeaORM 的模块化 Web 服务脚手架（cargo-generate template）。用户 `cargo generate` 后只维护 `src/modules/` 下的业务模块，框架层（`ws-core/`、`src/main.rs`）随 template 更新，用户不直接修改。

---

## 目录职责边界

| 路径 | 职责 | 用户是否修改 |
|---|---|---|
| `ws-core/` | 框架基础设施：AppModule trait、AppState、AppError、config 工具（独立 crate） | 否 |
| `src/modules/mod.rs` | 模块注册表，`all_modules()` 返回所有模块 | **是，唯一必须改的框架侧文件** |
| `src/modules/<name>/` | 业务模块，每个子目录完全独立 | 是 |
| `src/main.rs` | 启动入口：logging、schema sync、模块初始化、router 组装 | 否 |
| `config/` | TOML 配置文件，运行时读取，首次启动自动生成 | 是（运维配置） |
| `example/` | 独立示例项目，不参与主项目编译，cargo generate 时排除 | 参考用 |

---

## 核心 API

### AppModule trait（`ws-core/src/module.rs`）

```rust
pub trait AppModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;        // 模块名，同时是路由前缀
    fn routes(&self) -> Router<AppState>;  // 模块路由
    fn init<'a>(&'a self, _state: &'a AppState)
        -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>
    {
        Box::pin(async { Ok(()) })  // 默认空实现
    }
}
```

不使用 `async_trait`，通过手动 `Pin<Box<dyn Future>>` 保持 dyn-compatible。

### AppState（`ws-core/src/state.rs`）

```rust
pub struct AppState {
    pub core_config: CoreConfig,
    pub db: DatabaseConnection,          // sea-orm，内部是 Arc，clone 廉价
    extensions: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl AppState {
    pub fn set_module<T: Send + Sync + 'static>(&self, val: T);
    pub fn get_module<T: Send + Sync + 'static>(&self) -> Option<Arc<T>>;
}
```

### ModuleExt extractor（`ws-core/src/extract.rs`）

```rust
pub struct ModuleExt<T>(pub Arc<T>);
// impl FromRequestParts<AppState>，从 AppState::get_module 取值
// 取不到时返回 AppError::NotFound
```

### AppError（`ws-core/src/error.rs`）

变体：`InvalidToken`、`PermissionDenied`、`TooManySubmit`、`NotFound`、`InvalidField { field, reason }`、`DatabaseError(DbErr)`。

模块私有错误通过 `impl From<ModuleError> for AppError` 转换，路由函数用 `?` 传播。

---

## 模块结构约定

```
src/modules/<name>/
├── mod.rs       定义 <Name>Module struct，impl AppModule（必需）
├── routes.rs    路由函数，返回 Router<AppState>（必需）
├── entity/      sea-orm entity，带 #[sea_orm::model] 宏（可选）
├── service.rs   业务逻辑，derive Clone，通过 Extension layer 注入（可选）
├── error.rs     模块私有错误，impl From<_> for AppError（可选）
└── config.rs    模块私有配置，用 get_config("<name>") 读取（可选）
```

### 服务注入方式

`init` 调用 `state.set_module(...)` 注册服务，路由函数用 `ModuleExt<T>` extractor 提取：

```rust
// init 中注册（mod.rs）
fn init<'a>(&'a self, state: &'a AppState) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
    Box::pin(async move {
        state.set_module(MyService::new(state.db.clone()));
        Ok(())
    })
}

// handler 中消费（routes.rs）
async fn handler(ModuleExt(svc): ModuleExt<MyService>, ...) -> Result<...> { ... }
```

`service.rs` 中的 service struct 必须 `derive Clone`（`set_module` 存入 `Arc<T>`，handler 每次得到 `Arc<T>`）。

### 模块注册顺序

`all_modules()` 中的顺序即 `init()` 的执行顺序。若模块 B 的 `init` 或路由依赖模块 A 注册的服务，A 必须排在 B 前面。取不到服务时 `ModuleExt` 返回 `AppError::NotFound`。

---

## 数据库（Entity First）

- 使用 sea-orm 2.x entity-first 模式，**不存在 migration crate**
- entity 文件带 `#[sea_orm::model]` 宏，编译时自动注册到全局 inventory
- 启动时 `get_schema_registry("{{project-name}}::modules::*").sync(&db)` 增量同步 schema
- schema sync 只增不删（表、列、外键），删除 index 除外

---

## 错误处理规则

- **`init` 及启动阶段**：`expect`/`panic`，让进程尽早退出，错误信息最直接
- **路由 handler**：必须返回 `Result<T, AppError>`，用 `?` 传播，禁止 `unwrap`

---

## 禁止事项

- 不在 `ws-core/` 中 import 任何 `src/modules/` 的类型
- 不跨模块直接调用函数（通过 `state.set_module`/`ModuleExt<T>` 通信）
- 不在 `modules/mod.rs` 之外注册模块
- 不在 entity 文件中放业务逻辑
- 不在路由 handler 中 `panic`/`unwrap`

---

## 依赖说明

主要依赖及其用途：

| crate | 用途 |
|---|---|
| `axum 0.8` | HTTP 框架 |
| `sea-orm 2.0.0-rc.40` | ORM，features: `schema-sync`, `entity-registry` |
| `dashmap` | AppState 的 module extension map（线程安全 HashMap） |
| `tower-http` | middleware：trace、cors、catch-panic |
| `axum-server` | 支持 TLS 的服务器绑定 |
| `serde-inline-default` | 配置结构体字段默认值 |
| `shadow-rs` | 编译时注入 git/build 信息（ping 模块用） |
| `diff` | 配置文件 save_changes 时保留注释的 diff 合并 |
