use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityTrait, SchemaBuilder};
use tracing::log::LevelFilter;

use crate::config::core::CoreConfig;
use crate::config::get_config;

type RegisterFn = Box<dyn FnOnce(SchemaBuilder) -> SchemaBuilder + Send>;

#[derive(Clone)]
pub struct AppState {
    pub core_config: CoreConfig,
    pub db: DatabaseConnection,
    extensions: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    pending_entities: Arc<Mutex<Vec<RegisterFn>>>,
}

impl AppState {
    pub async fn new() -> Self {
        let core_config: CoreConfig = get_config("core");

        let mut opt = ConnectOptions::new(&core_config.db_uri);
        opt.sqlx_logging(true);
        opt.sqlx_logging_level(LevelFilter::Info);

        let db = Database::connect(opt)
            .await
            .unwrap_or_else(|e| panic!("Failed to connect to database '{}': {}", core_config.db_uri, e));

        Self {
            core_config,
            db,
            extensions: Arc::new(DashMap::new()),
            pending_entities: Arc::new(Mutex::new(Vec::new())),
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

    /// Queue an entity for schema sync. Called from module `init`.
    pub fn register_entity<E: EntityTrait + 'static>(&self) {
        self.pending_entities
            .lock()
            .unwrap()
            .push(Box::new(|builder: SchemaBuilder| builder.register(E::default())));
    }

    /// Run schema sync for all registered entities. Called once from `main` after all `init`.
    pub async fn sync_schema(&self) {
        let fns = std::mem::take(&mut *self.pending_entities.lock().unwrap());
        if fns.is_empty() {
            return;
        }
        let builder = fns.into_iter().fold(self.db.get_schema_builder(), |b, f| f(b));
        builder.sync(&self.db).await.expect("schema sync failed");
    }
}
