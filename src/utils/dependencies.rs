use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use tokio::sync::Mutex;

use crate::{
    types::{error::AppError, traits::object_store::ObjectStore},
    utils::cassandra::DbSession,
};

pub struct DependencyManager {
    object_stores: HashMap<String, Arc<dyn ObjectStore>>,
    db_sessions: HashMap<String, Arc<DbSession>>,
}

static DEPENDENCIES: OnceLock<Arc<Mutex<DependencyManager>>> = OnceLock::new();

pub fn dependencies() -> &'static Arc<Mutex<DependencyManager>> {
    DEPENDENCIES.get_or_init(|| Arc::new(Mutex::new(DependencyManager::new())))
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            object_stores: HashMap::new(),
            db_sessions: HashMap::new(),
        }
    }

    pub fn get_object_store(&self, key: &str) -> Result<Arc<dyn ObjectStore>, AppError> {
        Ok(self
            .object_stores
            .get(key)
            .cloned()
            .ok_or(AppError::MissingDependency(key.to_string()))?)
    }

    pub fn set_object_store(
        &mut self,
        key: &str,
        store: Arc<dyn ObjectStore>,
    ) -> Result<(), AppError> {
        self.object_stores.insert(key.into(), store);

        Ok(())
    }

    pub fn get_db_session(&self, key: &str) -> Result<Arc<DbSession>, AppError> {
        Ok(self
            .db_sessions
            .get(key)
            .cloned()
            .ok_or(AppError::MissingDependency(key.to_string()))?)
    }

    pub fn set_db_session(&mut self, key: &str, session: Arc<DbSession>) -> Result<(), AppError> {
        self.db_sessions.insert(key.into(), session);

        Ok(())
    }
}
