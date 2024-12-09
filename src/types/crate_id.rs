use crate::prelude::*;
use mongodb::bson::to_bson;
use mongodb::bson::Bson;
use semver::Version;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// A specified name and version of a crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct CrateId {
	pub name: String,
	// ts-rs represents versions as strings
	pub version: Version,
}


impl CrateId {
	pub fn new(name: impl Into<String>, version: Version) -> Self {
		Self {
			name: name.into(),
			version,
		}
	}
	pub fn into_scene_id(&self, project_name: impl Into<String>) -> SceneId {
		SceneId::new(self.clone(), project_name)
	}

	/// String in format `crate_name/version`
	pub fn path(&self) -> String { format!("{}/{}", self.name, self.version) }

	/// String in format `crates.io/crate_name/version`
	pub fn into_doc_id(&self) -> DocId {
		DocId(format!("crates.io/{}/{}", self.name, self.version))
	}
}
impl Into<Bson> for CrateId {
	fn into(self) -> Bson { to_bson(&self).expect("CrateId to Bson failed") }
}


impl std::fmt::Display for CrateId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}/{}", self.name, self.version)
	}
}

// #[cfg(test)]
impl CrateId {
	pub fn bevyhub_template() -> Self {
		let version = CargoManifest::bevyhub_crate_version();
		Self::new("bevyhub_template", version)
	}
	pub fn bevyhub_template_bad_version() -> Self {
		Self::new("bevyhub_template", Version::new(0, 0, 0))
	}
}
