//! Database connection builder and options.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ConnBuilder {
    pub db_path: PathBuf,
    pub create_if_missing: bool,
    pub max_open_files: i32,
    pub parallel_threads: usize,
}

impl ConnBuilder {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            create_if_missing: true,
            max_open_files: 512,
            parallel_threads: 4,
        }
    }

    pub fn with_create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
        self
    }

    pub fn with_max_open_files(mut self, files: i32) -> Self {
        self.max_open_files = files;
        self
    }

    pub fn with_parallel_threads(mut self, threads: usize) -> Self {
        self.parallel_threads = threads;
        self
    }
}
