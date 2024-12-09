use crate::prelude::*;
use anyhow::Result;
use rand::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// A scene document that is stored in a [DocumentDb].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SceneDoc {
	/// The unique database id in the form of stringified [SceneId],
	/// `{crate_name}/{scene_name}/{version}`
	_id: DocId,
	/// The crate name, crate version and scene id
	pub scene_id: SceneId,
	/// scene description or crate description or `{name} scene`
	pub description: String,
	/// Epoch timestamp
	#[ts(type = "number")]
	pub created_ms: u64,
	/// Optional thumbnail
	pub thumbnail: SceneThumb,
	/// Tree of scenes to include
	pub scene_include_tree: SceneIncludeTree,
	/// Optional app binary
	pub app: Option<SceneApp>,
	/// Optional link to a repository
	pub repository: Option<String>,
	/// Specifies whether this scene is in the latest version of the crate, defaults to false
	pub is_latest: bool,
	pub replication_config: ReplicationConfig,
}

impl HasDocId for SceneDoc {
	fn doc_id(&self) -> DocId { self._id.clone() }
}

impl SceneDoc {
	pub async fn from_manifest(
		api: &Services,
		crate_doc: &CrateDoc,
		cargo_lock: &CargoLock,
		crate_id: &CrateId,
		scenes: &ManifestMetadata,
		scene: &ManifestScene,
	) -> Result<Self> {
		let tree = SceneIncludeTree::from_manifest(
			api, cargo_lock, crate_id, scenes, scene,
		)
		.await?;

		let app = if let Some(app) = scene.app.as_ref() {
			Some(
				SceneApp::from_manifest(
					api, cargo_lock, crate_id, scenes, scene, app,
				)
				.await?,
			)
		} else {
			None
		};

		let scene_id = crate_id.into_scene_id(&scene.name);

		Ok(Self {
			_id: scene_id.into_doc_id(),
			scene_id,
			thumbnail: SceneThumb::from_manifest(&scene),
			description: scene.description.clone().unwrap_or_else(|| {
				crate_doc
					.description
					.clone()
					.unwrap_or_else(|| format!("The {} scene", scene.name))
			}),
			created_ms: epoch_millis(),
			scene_include_tree: tree,
			app,
			repository: crate_doc.repository.clone(),
			is_latest: false,
			replication_config: ReplicationConfig::from_manifest(
				&scene.replication,
			),
		})
	}
}

fn epoch_millis() -> u64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_millis() as u64
}


// js friendly enum for binary type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
pub enum SceneThumb {
	#[serde(rename = "url")]
	Url { url: String },
	#[serde(rename = "text")]
	Text { text: String },
}

impl SceneThumb {
	pub fn from_manifest(manifest: &ManifestScene) -> Self {
		if let Some(url) = &manifest.thumb_url {
			SceneThumb::Url { url: url.clone() }
		} else if let Some(text) = &manifest.thumb_text {
			SceneThumb::Text { text: text.clone() }
		} else {
			let random_emojis = vec![
				"🌈", "🌟", "🎉", "🎨", "🚀", "🌌", "🌞", "🌊", "🌸", "🌻",
			];
			let random_emoji = random_emojis
				.choose(&mut rand::thread_rng())
				.unwrap_or(&"🎨");

			SceneThumb::Text {
				text: random_emoji.to_string(),
			}
		}
	}
}


#[cfg(test)]
impl SceneDoc {
	pub async fn bevyhub_template_my_beautiful_scene() -> Result<Self> {
		let id = CrateId::bevyhub_template();
		let api = Services::init().await?;
		let (_, scenes) = api.unpack_crate_to_db(&id).await?;
		return Ok(scenes[2].clone());
	}
}
