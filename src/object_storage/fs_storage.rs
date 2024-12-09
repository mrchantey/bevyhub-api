use crate::prelude::*;
use anyhow::Result;
use axum::body::Bytes;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;

/// Mock object storage, using fs
#[derive(Default, Clone)]
pub struct FsStorage;


impl FsStorage {
	const BASE_PATH: &'static str = "target/storage";
	fn path(&self, key: &str) -> PathBuf {
		let mut path = PathBuf::from(Self::BASE_PATH);
		path.push(key);
		path
	}
}
#[async_trait::async_trait]
impl ObjectStorage for FsStorage {
	async fn get(&self, key: &str) -> Result<Bytes> {
		let path = self.path(key);
		let bytes = fs::read(path).await?;
		Ok(bytes.into())
	}
	async fn list(&self, prefix: &str) -> Result<Vec<StorageObjectInfo>> {
		// let path = self.path(prefix);
		let required_prefix = self.path(prefix);
		let required_prefix = required_prefix
			.to_str()
			.ok_or_else(|| anyhow::anyhow!("Invalid prefix"))?;
		let objs = read_dir_recursive(Self::BASE_PATH)
			.into_iter()
			.map(|path| path.as_path().to_string_lossy().to_string())
			.filter(|path| path.starts_with(required_prefix))
			.map(|path| StorageObjectInfo::new_no_creation_date(path))
			.collect();
		Ok(objs)
	}


	async fn put(&self, key: &str, value: Bytes) -> Result<()> {
		let path = self.path(key);
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent).await?;
		}
		fs::write(path, value).await?;
		Ok(())
	}

	async fn delete(&self, key: &str) -> Result<()> {
		let path = self.path(key);
		fs::remove_file(path).await?;
		Ok(())
	}

	async fn exists(&self, key: &str) -> Result<bool> {
		let path = self.path(key);
		Ok(File::open(&path).await.is_ok())
	}
}


/// Get all _files_ in a directory recursively
fn read_dir_recursive(path: impl Into<PathBuf>) -> Vec<PathBuf> {
	read_dir_recursive_inner(Vec::new(), path.into())
}
fn read_dir_recursive_inner(
	mut acc: Vec<PathBuf>,
	path: PathBuf,
) -> Vec<PathBuf> {
	if !path.is_dir() {
		acc.push(path);
		return acc;
	}
	let children = std::fs::read_dir(&path).unwrap();
	children
		.filter_map(|c| c.ok().map(|c| c.path()))
		.fold(acc, read_dir_recursive_inner)
}



#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use axum::body::Bytes;
	use fs_storage::read_dir_recursive;
	use sweet::*;

	#[tokio::test]
	async fn works() -> Result<()> {
		let storage = FsStorage::default();
		let key = "foo.txt";
		let value = Bytes::from("bar");
		storage.delete(key).await.ok();
		expect(storage.exists(key).await?).to_be_false()?;

		storage.put(key, value.clone()).await?;

		let list = storage.list("fo").await?;
		// println!("{:?}", list);
		expect(&list).any(|other| other.name.contains(key))?;
		let list = storage.list("bar").await?;
		expect(&list).not().any(|other| other.name.contains(key))?;


		let bytes = storage.get(key).await?;
		expect(bytes).to_be(value)?;

		expect(storage.exists(key).await?).to_be_true()?;
		storage.delete(key).await.ok();
		expect(storage.exists(key).await?).to_be_false()?;

		Ok(())
	}

	#[tokio::test]
	async fn test_read_dir_recursive() -> Result<()> {
		let path = "src";
		let paths = read_dir_recursive(path);
		expect(&paths).not().to_be_empty()?;
		expect(paths.len()).to_be_greater_than(10)?;
		// for path in paths.iter() {
		// 	expect(path.to_string_lossy().ends_with(".rs")).to_be_true()?;
		// }
		Ok(())
	}
}
