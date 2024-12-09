use crate::prelude::*;
use anyhow::Result;
use axum::body::Bytes;
use semver::Version;
use serde::Deserialize;

/// Trait for getting the Cargo.toml of crates
/// Can be implemented for Crates.io api or mocked
#[async_trait::async_trait]
pub trait CargoRegistry: 'static + Send + Sync {
	/// A sorted (lowest to highest) list of unyanked versions for a crate.
	/// The latest version is last, ie `versions[versions.len() - 1]`
	async fn versions(&self, crate_name: &str) -> Result<Vec<Version>> {
		let index = self.crate_index(crate_name).await?;
		let mut versions: Vec<Version> = index
			.into_iter()
			.filter(|v| !v.yanked)
			.map(|v| Version::parse(&v.vers))
			.collect::<Result<_, _>>()?;

		versions.sort();

		Ok(versions)
	}

	async fn version_or_latest(
		&self,
		crate_name: &str,
		version: &str,
	) -> Result<Version> {
		if version == "latest" {
			self.latest_version(crate_name).await
		} else {
			Ok(Version::parse(version)?)
		}
	}

	/// Return the latest version
	async fn latest_version(&self, crate_name: &str) -> Result<Version> {
		let versions = self.versions(crate_name).await?;
		match versions.last() {
			Some(v) => Ok(v.clone()),
			None => {
				Err(anyhow::anyhow!("No versions found for {}", crate_name))
			}
		}
	}

	async fn crate_index(&self, crate_name: &str) -> Result<CrateIndex>;

	// fn get(&mut self, crate_name: &str, version: &str);
	// fn get_latest(&mut self, crate_name: &str);
	async fn tarball(&self, crate_id: &CrateId) -> Result<Bytes>;
}

#[derive(Clone)]
pub enum CargoRegistryEnum {
	Cached(LocalCacheRegistry),
	CratesIo(CratesIo),
}

impl CargoRegistryEnum {
	pub fn new(env: ApiEnvironment) -> Result<Self> {
		match env {
			ApiEnvironment::Local => {
				Ok(Self::Cached(LocalCacheRegistry::default()))
			}
			ApiEnvironment::Staging => {
				Ok(Self::CratesIo(CratesIo::default()))
				// Ok(Self::Cached(LocalCacheRegistry::read_only()))
			}
			ApiEnvironment::Prod => Ok(Self::CratesIo(CratesIo::default())),
		}
	}

	pub fn inner(&self) -> &dyn CargoRegistry {
		match self {
			CargoRegistryEnum::Cached(val) => val,
			CargoRegistryEnum::CratesIo(val) => val,
		}
	}
}

pub type CrateIndex = Vec<CrateIndexVersion>;

/// Raw value from crates.io
#[derive(Debug, Deserialize)]
pub struct CrateIndexVersion {
	pub name: String,
	pub yanked: bool,
	pub vers: String,
	pub deps: Vec<CrateIndexDep>,
	pub cksum: String,
	// pub features: Vec<String>,
}

/// Raw value from crates.io
#[derive(Debug, Deserialize)]
pub struct CrateIndexDep {
	pub name: String,
	pub req: String,
	// pub features: Vec<String>,
	pub optional: bool,
	pub default_features: bool,
	// pub target: Option<String>,
	pub kind: String,
}
