use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use tokio::sync::Mutex;

use crate::types::{error::AppError, traits::object_store::ObjectStore};

pub struct DependencyManager {
    object_stores: HashMap<String, Arc<dyn ObjectStore>>,
}

static DEPENDENCIES: OnceLock<Arc<Mutex<DependencyManager>>> = OnceLock::new();

pub fn dependencies() -> &'static Arc<Mutex<DependencyManager>> {
    DEPENDENCIES.get_or_init(|| Arc::new(Mutex::new(DependencyManager::new())))
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            object_stores: HashMap::new(),
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
}
