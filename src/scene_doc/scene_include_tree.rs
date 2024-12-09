use crate::prelude::*;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// Tree of scenes to be included, specifying exact versions and paths
/// a bit like a `Cargo.lock`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct SceneIncludeTree {
	/// The path of the unpkged scene
	pub file: SceneFile,
	pub scene_id: SceneId,
	pub children: Vec<SceneIncludeTree>,
}

/// Js friendly enum for a scene type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(tag = "kind")]
pub enum SceneFile {
	#[serde(rename = "json")]
	Json { path: String },
	#[serde(rename = "ron")]
	Ron { path: String },
	#[serde(rename = "bsn")]
	Bsn {
		/// Id of the scene this app belongs to
		/// Path relative to _published_ crate root
		path: String,
	},
	#[serde(rename = "inline_json")]
	InlineJson { json: String },
}

impl SceneFile {
	pub fn from_manifest(manifest: &ManifestScene) -> Result<Self> {
		if let Some(scene_json) = &manifest.scene_json {
			Ok(Self::InlineJson {
				json: scene_json.clone(),
			})
		} else if let Some(path) = &manifest.path {
			match path.split('.').last() {
				Some("json") => Ok(Self::Json {
					path: manifest.path.clone().unwrap(),
				}),
				Some("ron") => Ok(Self::Ron {
					path: manifest.path.clone().unwrap(),
				}),
				Some("bsn") => Ok(Self::Bsn {
					path: manifest.path.clone().unwrap(),
				}),
				_ => {
					anyhow::bail!("Invalid scene file extension: {}", path)
				}
			}
		} else {
			Ok(Self::Json {
				path: format!("scenes/{}.json", manifest.name).into(),
			})
		}
	}
}


impl SceneIncludeTree {
	pub fn new(
		scene_id: SceneId,
		scene: SceneFile,
		children: Vec<SceneIncludeTree>,
	) -> Self {
		Self {
			scene_id,
			file: scene,
			children,
		}
	}

	pub async fn from_manifest(
		api: &Services,
		cargo_lock: &CargoLock,
		manifest_crate_id: &CrateId,
		manifest_metadata: &ManifestMetadata,
		manifest_scene: &ManifestScene,
	) -> Result<Self> {
		let scene_file = SceneFile::from_manifest(manifest_scene)?;
		let dependencies = Self::build_dependencies(
			api,
			cargo_lock,
			manifest_crate_id,
			manifest_metadata,
			&manifest_scene.get_includes(),
		)
		.await?;
		let id = manifest_crate_id.into_scene_id(&manifest_scene.name);
		Ok(Self::new(id, scene_file, dependencies))
	}

	pub async fn build_dependencies(
		api: &Services,
		cargo_lock: &CargoLock,
		manifest_crate_id: &CrateId,
		manifest_metadata: &ManifestMetadata,
		deps: &Vec<ManifestDependency>,
	) -> Result<Vec<Self>> {
		let futs = deps.iter().map(|dep| {
			Self::build_dependency(
				api,
				cargo_lock,
				manifest_crate_id,
				manifest_metadata,
				dep,
			)
		});
		futures::future::try_join_all(futs).await
	}



	async fn build_dependency(
		api: &Services,
		cargo_lock: &CargoLock,
		manifest_crate_id: &CrateId,
		manifest_metadata: &ManifestMetadata,
		dep: &ManifestDependency,
	) -> Result<Self> {
		let (crate_name, scene_name) =
			dep.into_crate_and_scene(&manifest_crate_id.name)?;
		if crate_name == manifest_crate_id.name {
			let sibling_scene = manifest_metadata.find_scene(&scene_name)?;

			return Self::from_manifest(
				api,
				cargo_lock,
				manifest_crate_id,
				manifest_metadata,
				sibling_scene,
			)
			.await;
		} else {
			let external_crate_id = cargo_lock.crate_id(&crate_name)?;
			let scene_id = SceneId::new(external_crate_id, &scene_name);
			let scene_doc = api.scene_doc(&scene_id).await?;

			Ok(scene_doc.scene_include_tree)
		}
	}
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use sweet::*;

	#[tokio::test]
	async fn works() -> Result<()> {
		let api = Services::init().await?;

		let scene = api.scene_doc(&SceneId::my_beautiful_scene()).await?;
		let tree = &scene.scene_include_tree;

		expect(tree.children.len()).to_be(2)?;
		expect(&tree.scene_id).to_be(&SceneId::my_beautiful_scene())?;

		// let str = scene_tree::SceneFile::export_to_string();

		expect(&tree.file).to_be(&SceneFile::Json {
			path: "scenes/my-beautiful-scene.json".into(),
		})?;
		// expect(&tree.children[0].scene).to_be(&"simple_environment".into())?;
		// expect(&scene.name).to_be(&"simple_scene".into())?;

		Ok(())
	}
}
