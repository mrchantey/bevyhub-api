use crate::prelude::*;
use anyhow::Result;
use axum::body::Bytes;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

/// Trait for storing and retrieving binary blobs,
/// implemented by [S3Storage] and [`FsStorage`]
#[async_trait::async_trait]
pub trait ObjectStorage: 'static + Send + Sync {
	async fn get(&self, key: &str) -> Result<Bytes>;
	/// List all objects with the given prefix.
	/// Some implementations, ie [FsStorage] may only read the top level directory.
	async fn list(&self, prefix: &str) -> Result<Vec<StorageObjectInfo>>;
	async fn put(&self, key: &str, value: Bytes) -> Result<()>;
	async fn delete(&self, key: &str) -> Result<()>;
	/// Check if an object exists.
	/// # Errors
	/// May error if passed a directory.
	async fn exists(&self, key: &str) -> Result<bool>;


	async fn put_many(&self, values: Vec<(String, Bytes)>) -> Result<()> {
		let futs = values
			.into_iter()
			.map(|(key, value)| async move {
				let key = key.clone();
				self.put(&key, value).await
			})
			.collect::<Vec<_>>();
		let _ = futures::future::try_join_all(futs).await?;
		Ok(())
	}
}

#[derive(Clone)]
pub enum ObjectStorageEnum {
	Local(FsStorage),
	S3(S3Storage),
}

impl ObjectStorageEnum {
	pub async fn new(env: ApiEnvironment) -> Result<Self> {
		match env {
			ApiEnvironment::Local => Ok(Self::Local(FsStorage::default())),
			ApiEnvironment::Staging => Ok(Self::S3(S3Storage::new(env).await)),
			ApiEnvironment::Prod => Ok(Self::S3(S3Storage::new(env).await)),
		}
	}

	pub fn inner(&self) -> &dyn ObjectStorage {
		match self {
			ObjectStorageEnum::Local(val) => val,
			ObjectStorageEnum::S3(val) => val,
		}
	}
}




/// Description of an object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageObjectInfo {
	/// Path to the object
	pub name: String,
	/// Time since epoch
	pub created: Duration,
}


impl StorageObjectInfo {
	pub fn new(name: String, created: Duration) -> Self {
		Self { name, created }
	}
	pub fn new_no_creation_date(name: String) -> Self {
		Self {
			name,
			created: Duration::default(),
		}
	}
}
