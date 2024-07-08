use lazy_static::lazy_static;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log::LevelFilter;
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
}
