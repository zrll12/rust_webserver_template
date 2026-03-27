use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log::LevelFilter;
use crate::config::core::CoreConfig;
use crate::config::get_config;

pub struct AppState {
    pub core_config: CoreConfig,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new() -> Self {
        let core_config: CoreConfig = get_config("core");


        let mut opt = ConnectOptions::new(&core_config.db_uri);
        opt.sqlx_logging(true);
        opt.sqlx_logging_level(LevelFilter::Info);

        let db: DatabaseConnection = futures::executor::block_on(Database::connect(opt))
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to connect to database '{}': {}",
                    core_config.db_uri, e
                )
            });

        Self { core_config, db }
    }
}
