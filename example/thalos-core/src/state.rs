use std::any::{Any, TypeId};
use std::sync::Arc;
use dashmap::DashMap;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log::LevelFilter;
use crate::config::core::CoreConfig;
use crate::config::get_config;

#[derive(Clone)]
pub struct AppState {
    pub core_config: CoreConfig,
    pub db: DatabaseConnection,
    extensions: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
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

        Self {
            core_config,
            db,
            extensions: Arc::new(DashMap::new()),
        }
    }

    pub fn set_module<T: Send + Sync + 'static>(&self, val: T) {
        self.extensions.insert(TypeId::of::<T>(), Arc::new(val));
    }

    pub fn get_module<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|v| v.clone().downcast::<T>().ok())
    }
}
