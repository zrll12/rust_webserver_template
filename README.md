## Rust Web Backend Template

#### Usage: `cargo generate --git https://github.com/zrll12/rust_webserver_template.git`

### Architecture

```
.
├── migration/          # All your database schems (see https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/)
└── src/
    ├── main.rs         # Project entry
    ├── controller/     # Router
    └── config/         # Config (Every config file is defiened into a struct)
```

You can use 

``` rust
lazy_static! {
    static ref CORE_CONFIG: CoreConfig = get_config("core");
}
```

to load your config. 

If you have default key in your struct, it can generate files automatically
