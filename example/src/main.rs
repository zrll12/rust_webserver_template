use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::classify::StatusInRangeAsFailures;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::log::warn;
use tracing::{debug, info};
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, fmt};

mod modules;

#[tokio::main]
async fn main() {
    let state = ws_core::state::AppState::new();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&state.core_config.trace_level));
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .build("logs")
        .unwrap();
    let (non_blocking_appender, _guard) = non_blocking(file_appender);

    let formatting_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.f(%:z)".to_string()));
    let file_layer = fmt::layer()
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.f(%:z)".to_string()))
        .with_ansi(false)
        .with_writer(non_blocking_appender);
    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(file_layer)
        .init();

    // schema-sync: 根据 entity 定义自动建表/加列
    state
        .db
        .get_schema_registry("example::modules::*")
        .sync(&state.db)
        .await
        .expect("Schema sync failed");

    // 初始化所有模块（按声明顺序，模块在此调用 state.set_module(...)）
    let module_list = modules::all_modules();
    for m in &module_list {
        m.init(&state)
            .await
            .unwrap_or_else(|e| panic!("{} init failed: {}", m.name(), e));
    }

    // 组装路由：每个模块挂载到 /<name>
    let router = module_list.iter().fold(Router::new(), |r, m| {
        r.nest(&format!("/{}", m.name()), m.routes())
    });

    let origins = state
        .core_config
        .origins
        .iter()
        .map(|x| x.parse::<HeaderValue>().unwrap())
        .collect::<Vec<_>>();

    let app = router
        .with_state(state.clone())
        .layer(TraceLayer::new(
            StatusInRangeAsFailures::new(400..=599).into_make_classifier(),
        ))
        .layer(DefaultBodyLimit::max(
            state.core_config.max_body_size * 1024 * 1024,
        ))
        .layer(
            CorsLayer::very_permissive()
                .allow_origin(origins)
                .allow_credentials(state.core_config.allow_credentials),
        )
        .layer(CatchPanicLayer::new());

    let addr: SocketAddr = state.core_config.server_addr.parse().unwrap();
    info!("Listening: {addr}");

    if state.core_config.tls {
        debug!("HTTPS enabled.");
        let tls_config = RustlsConfig::from_pem_file(
            &state.core_config.ssl_cert,
            &state.core_config.ssl_key,
        )
        .await
        .unwrap();
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        warn!("HTTPS disabled.");
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
