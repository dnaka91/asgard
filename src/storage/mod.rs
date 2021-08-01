use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    pin::Pin,
};

use anyhow::{bail, Result};
use async_trait::async_trait;
use semver::Version;
use tokio::{
    fs::{self, File},
    io::AsyncRead,
};

use crate::models::CrateName;

type PinnedRead = Pin<Box<dyn AsyncRead + Send>>;

/// The storage service that manages storing and loading crate content. The content is the source
/// code tarball packaged by cargo and uploaded with a new release.
#[async_trait]
pub trait Service: Send + Sync + 'static {
    /// Store a new crate tarball in the storage with given name and version.
    async fn store(&self, name: &CrateName, version: &Version, data: &[u8]) -> Result<()>;
    /// Try to locate the crate data identified by name and version and open it for reading if it
    /// exists.
    async fn get(&self, name: &CrateName, version: &Version) -> Result<Option<PinnedRead>>;
}

/// Main implementation of the storage [`Service`].
struct ServiceImpl {
    location: PathBuf,
}

#[async_trait]
impl Service for ServiceImpl {
    async fn store(&self, name: &CrateName, version: &Version, data: &[u8]) -> Result<()> {
        let out = self.location.join(name.as_ref());

        fs::create_dir_all(&out).await?;

        let out = out.join(format!("{}-{}.crate", name, version));

        fs::write(out, data).await?;

        Ok(())
    }

    async fn get(&self, name: &CrateName, version: &Version) -> Result<Option<PinnedRead>> {
        let file_name = self
            .location
            .join(name.as_ref())
            .join(format!("{}-{}.crate", name, version));

        match File::open(file_name).await {
            Ok(f) => Ok(Some(Box::pin(f))),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => bail!(e),
        }
    }
}

/// Create a new storage service.
pub fn new(location: &Path) -> impl Service {
    ServiceImpl {
        location: location.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn service_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let service = new(dir.path());

        service
            .store(&"test".parse().unwrap(), &"1.0.0".parse().unwrap(), b"test")
            .await
            .unwrap();

        let reader = service
            .get(&"test".parse().unwrap(), &"1.0.0".parse().unwrap())
            .await
            .unwrap();

        assert!(reader.is_some());

        let reader = service
            .get(&"test".parse().unwrap(), &"2.0.0".parse().unwrap())
            .await
            .unwrap();

        assert!(reader.is_none());
    }
}
