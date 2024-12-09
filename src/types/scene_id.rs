use super::CrateId;
use crate::prelude::*;
use mongodb::bson::to_bson;
use mongodb::bson::Bson;
use semver::Version;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;



/// A specified name and version of a scene.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct SceneId {
	/// The crate that this scene belongs to
	pub crate_id: CrateId,
	/// The name of the scene
	pub scene_name: String,
}

impl Into<Bson> for SceneId {
	fn into(self) -> Bson { to_bson(&self).expect("SceneId to Bson failed") }
}


impl SceneId {
	pub fn new(crate_id: CrateId, scene_name: impl Into<String>) -> Self {
		Self {
			crate_id,
			scene_name: scene_name.into(),
		}
	}
	pub fn with_crate_name(
		crate_name: impl Into<String>,
		version: Version,
		scene_name: impl Into<String>,
	) -> Self {
		Self::new(CrateId::new(crate_name, version), scene_name)
	}
	/// Construct from `{crate_name}, {scene_name}, {version}`
	pub fn from_parts(
		crate_name: String,
		scene_name: String,
		version: Version,
	) -> Self {
		Self::new(CrateId::new(crate_name, version), scene_name)
	}
	pub fn scene_name(&self) -> &str { &self.scene_name }
	pub fn crate_id(&self) -> &CrateId { &self.crate_id }
	pub fn crate_name(&self) -> &str { &self.crate_id.name }
	pub fn version(&self) -> &Version { &self.crate_id.version }

	/// String in format `crate_name/version/scene_name`
	pub fn path(&self) -> String {
		format!("{}/{}", self.crate_id.path(), self.scene_name)
	}
	/// String in format `crates.io/crate_name/scene_name/version`
	pub fn into_doc_id(&self) -> DocId {
		DocId(format!(
			"crates.io/{}/{}/{}",
			self.crate_id.name, self.scene_name, self.crate_id.version
		))
	}
}


impl std::fmt::Display for SceneId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}/{}/{}",
			self.crate_id.name, self.scene_name, self.crate_id.version
		)
	}
}

#[cfg(test)]
impl SceneId {
	pub fn my_beautiful_scene() -> Self {
		Self::new(CrateId::bevyhub_template(), "my-beautiful-scene")
	}
}
