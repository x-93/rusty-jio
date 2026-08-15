use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

pub struct FileAppender {
    file: Mutex<File>,
}

impl FileAppender {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: Mutex::new(file),
        })
    }

    pub fn write_log(&self, msg: &str) -> io::Result<()> {
        let mut f = self.file.lock().unwrap();
        writeln!(f, "{msg}")?;
        f.flush()
    }
}
