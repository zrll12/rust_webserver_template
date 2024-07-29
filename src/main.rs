use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum_server::tls_rustls::RustlsConfig;
use lazy_static::lazy_static;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::classify::StatusInRangeAsFailures;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing::log::{LevelFilter, warn};
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, Registry};
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use migration::{Migrator, MigratorTrait};
use crate::config::core::{CoreConfig};
use crate::config::get_config;

mod config;
mod controller;

lazy_static! {
    static ref CORE_CONFIG: CoreConfig = get_config("core");
    static ref DATABASE: DatabaseConnection = {
        let mut opt = ConnectOptions::new(&CORE_CONFIG.db_uri);
        opt.sqlx_logging(true);
        opt.sqlx_logging_level(LevelFilter::Info);
        futures::executor::block_on(Database::connect(opt)).unwrap_or_else(|e| {
            panic!("Failed to connect to database '{}': {}", CORE_CONFIG.db_uri, e)
        })
    };
}

#[tokio::main]
async fn main() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&CORE_CONFIG.trace_level));
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .filename_prefix("logs/")
        .build("")
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

    Migrator::up(&*DATABASE, None).await.unwrap();

    let origins = CORE_CONFIG.origins.clone().iter().map(|x| x.parse().unwrap()).collect::<Vec<HeaderValue>>();
    let app = controller::all_routers()
        .layer(TraceLayer::new(
            StatusInRangeAsFailures::new(400..=599).into_make_classifier()
        ))
        .layer(DefaultBodyLimit::max(
            CORE_CONFIG.max_body_size * 1024 * 1024,
        ))
        .layer(CorsLayer::very_permissive().allow_origin(origins).allow_credentials(CORE_CONFIG.allow_credentials))
        .layer(CatchPanicLayer::new());

    let addr = CORE_CONFIG.server_addr.parse().unwrap();
    info!("Listening: {addr}");

    if CORE_CONFIG.tls {
        debug!("HTTPS enabled.");
        let tls_config =
            RustlsConfig::from_pem_file(&CORE_CONFIG.ssl_cert, &CORE_CONFIG.ssl_key)
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
