use std::path::Path;

use crate::{KvsEngine, KvsError, Result};

#[derive(Clone)]
pub struct SledKvsEngine {
    db: sled::Db,
}

impl SledKvsEngine {
    pub fn open<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path> + std::fmt::Debug,
    {
        Ok(SledKvsEngine {
            db: sled::open(&path)?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> crate::Result<Option<String>> {
        Ok(self
            .db
            .get(key)?
            .map(|v| v.as_ref().to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn set(&mut self, key: String, value: String) -> crate::Result<()> {
        self.db.insert(key, value.as_bytes()).map(|_| ())?;
        self.db.flush()?;
        Ok(())
    }

    fn remove(&mut self, key: String) -> crate::Result<()> {
        self.db.remove(&key)?.ok_or(KvsError::KeyNotFound(key))?;
        self.db.flush()?;
        Ok(())
    }
}
