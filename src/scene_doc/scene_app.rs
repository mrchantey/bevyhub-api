use crate::prelude::*;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// A resolved form of a scene app specified in the `Cargo.toml`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SceneApp {
	pub binary: BevyBinary,
	pub replication_registry_url: Option<String>,
	pub type_registry_url: Option<String>,
}

impl SceneApp {
	pub async fn from_manifest(
		api: &Services,
		cargo_lock: &CargoLock,
		manifest_crate_id: &CrateId,
		manifest_metadata: &ManifestMetadata,
		scene: &ManifestScene,
		app: &ManifestApp,
	) -> Result<Self> {
		match app {
			ManifestApp::Wasm {
				js_url, wasm_url, ..
			} => Ok(Self {
				binary: BevyBinary::Wasm {
					scene_id: manifest_crate_id.into_scene_id(&scene.name),
					js_url: js_url.clone(),
					wasm_url: wasm_url.clone(),
					canvas_id: app.canvas_id(),
				},
				replication_registry_url: app.replication_registry_url(),
				type_registry_url: app.type_registry_url(),
				// replication: ReplicationConfig::from_manifest(
				// 	&app.replication_registry_url(),
				// ),
			}),
			app => {
				let (crate_name, scene_name) =
					app.into_crate_and_scene(&manifest_crate_id.name)?;

				if crate_name == manifest_crate_id.name {
					let sibling_scene =
						manifest_metadata.find_scene(&scene_name)?;
					let scene_id = manifest_crate_id.into_scene_id(scene_name);
					// if we depended on another scene it shouldnt be None
					let sibling_app =
						sibling_scene.app.as_ref().ok_or_else(|| {
							anyhow::anyhow!(
								"App not found in scene: {}",
								scene_id
							)
						})?;

					Box::pin(Self::from_manifest(
						api,
						cargo_lock,
						manifest_crate_id,
						manifest_metadata,
						&sibling_scene,
						sibling_app,
					))
					.await
				} else {
					let crate_id = cargo_lock.crate_id(&crate_name)?;

					let scene = api
						.scene_doc(&SceneId::new(crate_id, &scene_name))
						.await?;

					// if we depended on another scene it shouldnt be None
					let app = scene.app.to_owned().ok_or_else(|| {
						anyhow::anyhow!(
							"App not found in scene: {}",
							scene_name
						)
					})?;
					Ok(app)
				}
			}
		}
	}
}

// js friendly enum for binary type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
pub enum BevyBinary {
	#[serde(rename = "wasm")]
	Wasm {
		/// Id of the scene this app belongs to
		scene_id: SceneId,
		/// URL to the main.js file
		js_url: String,
		/// URL to the main_bg.wasm file
		wasm_url: String,
		/// The canvas id that this app expects to render to
		canvas_id: Option<String>,
	},
}

impl BevyBinary {}
