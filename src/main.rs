use crate::app_state::AppState;
use crate::config::core::CoreConfig;
use crate::config::get_config;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum_server::tls_rustls::RustlsConfig;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::net::SocketAddr;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::classify::StatusInRangeAsFailures;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::log::{LevelFilter, warn};
use tracing::{debug, info};
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, fmt};

mod app_state;
mod config;
mod controller;
mod error;

#[tokio::main]
async fn main() {
    let app_state = AppState::new();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&app_state.core_config.trace_level));
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

    Migrator::up(&app_state.db, None).await.unwrap();

    let origins = app_state
        .core_config
        .origins
        .clone()
        .iter()
        .map(|x| x.parse().unwrap())
        .collect::<Vec<HeaderValue>>();
    let app = controller::all_routers()
        .layer(TraceLayer::new(
            StatusInRangeAsFailures::new(400..=599).into_make_classifier(),
        ))
        .layer(DefaultBodyLimit::max(
            app_state.core_config.max_body_size * 1024 * 1024,
        ))
        .layer(
            CorsLayer::very_permissive()
                .allow_origin(origins)
                .allow_credentials(app_state.core_config.allow_credentials),
        )
        .layer(CatchPanicLayer::new());

    let addr: SocketAddr = app_state.core_config.server_addr.parse().unwrap();
    info!("Listening: {addr}");

    if app_state.core_config.tls {
        debug!("HTTPS enabled.");
        let tls_config = RustlsConfig::from_pem_file(
            &app_state.core_config.ssl_cert,
            &app_state.core_config.ssl_key,
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
