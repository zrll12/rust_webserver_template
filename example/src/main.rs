use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum_server::Handle;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::time::Duration;
use thalos_core::module::ModuleEntry;
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
#[cfg(all(test, feature = "openapi"))]
mod openapi_export;

#[cfg(feature = "openapi")]
static OPENAPI_JSON: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() {
    let state = thalos_core::state::AppState::new();

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

    // init modules
    let module_list = modules::all_modules();
    for e in &module_list {
        e.module
            .init(&state)
            .unwrap_or_else(|err| panic!("{} init failed: {}", e.module.name(), err));
    }

    // build router
    #[allow(unused_mut)]
    let mut router = module_list.iter().fold(Router::new(), |r, e| {
        r.nest(&format!("/{}", e.prefix), e.module.routes())
    });

    // build openapi feature
    #[cfg(feature = "openapi")]
    {
        use utoipa::openapi::InfoBuilder;
        use axum::routing::get;
        use axum::response::IntoResponse;

        let info = InfoBuilder::new()
            .title(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .build();
        let openapi = thalos_core::openapi::merge_modules(&module_list, info);
        let json = openapi.to_json().expect("OpenAPI serialization failed");
        let json_str: &'static str = OPENAPI_JSON.get_or_init(|| json);

        router = router.route(
            "/openapi.json",
            get(move || async move { (
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                json_str,
            ).into_response() }),
        );

        info!("OpenAPI spec available at /openapi.json");

        #[cfg(feature = "swagger-ui")]
        {
            use utoipa_swagger_ui::{Config, SwaggerUi};
            router = router.merge(
                SwaggerUi::new("/swagger-ui").config(Config::new(["/openapi.json"])),
            );
            info!("Swagger UI available at /swagger-ui");
        }
    }

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

    let handle = Handle::new();
    tokio::spawn(shutdown_handler(handle.clone(), module_list));

    if state.core_config.tls {
        debug!("HTTPS enabled.");
        let tls_config = RustlsConfig::from_pem_file(
            &state.core_config.ssl_cert,
            &state.core_config.ssl_key,
        )
        .await
        .unwrap();
        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        warn!("HTTPS disabled.");
        axum_server::bind(addr)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

async fn shutdown_handler(handle: Handle<SocketAddr>, module_list: Vec<ModuleEntry>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, draining connections...");
    handle.graceful_shutdown(Some(Duration::from_secs(30)));

    for e in module_list.iter().rev() {
        info!("Shutting down module: {}", e.module.name());
        e.module.shutdown();
    }

    info!("Shutdown complete.");
}
