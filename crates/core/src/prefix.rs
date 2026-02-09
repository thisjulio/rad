use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;

pub struct Prefix {
    pub root: PathBuf,
}

impl Prefix {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn initialize(&self) -> Result<()> {
        let dirs = [
            "system",
            "data",
            "data/app",
            "data/data",
            "dev",
            "proc",
            "sys",
            "tmp",
            "apex",
        ];

        for dir in dirs {
            let path = self.root.join(dir);
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
        }

        Ok(())
    }
}
