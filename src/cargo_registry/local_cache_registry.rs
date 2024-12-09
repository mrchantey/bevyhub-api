use super::cargo_registry::CargoRegistry;
use super::crates_io::CratesIo;
use crate::prelude::*;
use anyhow::Result;
use axum::body::Bytes;
use tokio::fs;

/// First attempts to load from fs before hitting crates.io
#[derive(Default, Clone)]
pub struct LocalCacheRegistry {
	crates_io: CratesIo,
	/// Only read from local, but dont write, useful for staging
	read_only: bool,
}

impl LocalCacheRegistry {
	// was used by staging, for staging tests we shouldnt write to the cache?
	pub fn read_only() -> Self {
		Self {
			crates_io: CratesIo::default(),
			read_only: true,
		}
	}
}

#[async_trait::async_trait]
impl CargoRegistry for LocalCacheRegistry {
	async fn crate_index(&self, crate_name: &str) -> Result<CrateIndex> {
		self.crates_io.crate_index(crate_name).await
	}

	async fn tarball(&self, crate_id: &CrateId) -> Result<Bytes> {
		let dir = "target/tarball-cache";
		let path =
			format!("{}/{}-{}.crate", dir, crate_id.name, crate_id.version);
		if let Ok(bytes) = fs::read(&path).await {
			return Ok(bytes.into());
		}
		println!("Local cache - downloading from registry: {}", path);
		let buff = self.crates_io.tarball(crate_id).await?;

		if !self.read_only {
			fs::create_dir_all(dir).await?;
			fs::write(&path, &buff).await?;
		}
		Ok(buff)
	}
}



#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use semver::Version;
	use sweet::*;
	
	#[tokio::test]
	async fn versions() -> Result<()> {
		let registry = LocalCacheRegistry::default();
		let versions = registry.versions("bevyhub_template").await?;
		expect(versions[0].to_string()).to_be("0.0.1-rc.1".to_string())?;
		Ok(())
	}
	#[tokio::test]
	async fn crate_index() -> Result<()> {
		let registry = LocalCacheRegistry::default();
		let index = registry.crate_index("bevyhub_template").await?;
		expect(index.len()).to_be_greater_or_equal_to(1)?;
		Ok(())
	}
	#[tokio::test]
	async fn tarball() -> Result<()> {
		let registry = LocalCacheRegistry::default();
		let tarball = registry
			.tarball(&CrateId::new(
				"bevyhub_template",
				Version::parse("0.0.1-rc.1").unwrap(),
			))
			.await?;
		expect(tarball.len()).to_be_greater_than(0)?;
		Ok(())
	}
}
