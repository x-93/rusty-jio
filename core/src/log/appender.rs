//! Console and file log appenders.

use std::path::Path;

pub struct LogAppender;

impl LogAppender {
    pub fn init_file_logger(_path: &Path) {
        // Initializes rotating file output
    }
}
