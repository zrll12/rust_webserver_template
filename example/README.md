# example

本目录是 rust_webserver_template 的完整示例项目，展示框架各层的典型用法。`cargo generate` 时此目录会被排除，不会进入生成的项目。

## workspace 结构

```
example/
├── ws-core/                    # 框架 core 层（与主项目 ws-core/ 保持同步）
│   └── src/
│       ├── error.rs            # AppError
│       ├── extract.rs          # ModuleExt<T> extractor
│       ├── module.rs           # AppModule trait
│       ├── state.rs            # AppState + set_module/get_module
│       └── config/             # CoreConfig + get_config()
├── src/
│   ├── main.rs                 # 启动入口
│   └── modules/
│       ├── mod.rs              # all_modules() 注册表
│       ├── ping/               # 内联模块示例（最简单形式）
│       ├── user/               # 完整内联模块示例（entity + service + extractor）
│       └── comment/            # 独立 crate 模块示例（含 Cargo.toml）
└── Cargo.toml                  # workspace 根
```

## 参考位置

| 想了解什么 | 看这里 |
|---|---|
| 最简模块结构 | `src/modules/ping/` |
| entity 定义（`#[sea_orm::model]`） | `src/modules/user/entity/user.rs` |
| service 注入（`state.set_module` / `ModuleExt<T>`） | `src/modules/user/mod.rs`, `routes.rs` |
| 自定义 extractor（token 鉴权） | `src/modules/user/extract.rs` + `service.rs` |
| 模块私有错误（`impl From<_> for AppError`） | `src/modules/user/error.rs` |
| 模块独立成 crate | `src/modules/comment/` |
| core 层实现细节 | `ws-core/src/` |

## 模块独立成 crate 的说明

`comment` 模块展示了如何把一个模块打包为独立 crate：

- 目录内有自己的 `Cargo.toml`，列为 workspace 成员
- 依赖 `ws-core`（path 依赖，位于 `ws-core/`）
- 在主项目的 `modules/mod.rs` 中像普通模块一样注册

**局限**：独立模块 crate 目前通过 path 依赖引用 ws-core，无法单独发布到 crates.io。若 ws-core 将来发布为独立 crate，替换为版本依赖即可解除此限制。

**关于 comment 模块未做 token 鉴权**：这是有意为之。`TokenInfo` 定义在主 crate 的 `user` 模块里，独立 crate 若依赖主 crate 会造成循环依赖（`example → comment → example`），所以 comment 无法直接使用 `TokenInfo`。

在真实项目中不会出现这个问题，因为需要鉴权的模块要么：
1. 以内联模块形式存在（直接 `use crate::modules::user::TokenInfo`，无循环）；
2. 或者已经独立成 crate，此时 `TokenInfo` 所在的 user crate 也是独立的，两个 crate 平行依赖即可。

结论：**只有"想独立成 crate 但依赖同项目内其他模块的业务类型"时才会遇到此问题**。解决方式是把共享类型提取到第三方 crate（如 `user-types`），或将相关模块全部独立成 crate。
