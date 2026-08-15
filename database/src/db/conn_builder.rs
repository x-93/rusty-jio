use crate::db::{JioDatabase, MemoryDatabase};
use std::sync::Arc;

pub struct ConnBuilder {
    path: Option<String>,
}

impl ConnBuilder {
    pub fn new() -> Self {
        Self { path: None }
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn build(self) -> Arc<dyn JioDatabase> {
        Arc::new(MemoryDatabase::new())
    }
}

impl Default for ConnBuilder {
    fn default() -> Self {
        Self::new()
    }
}
