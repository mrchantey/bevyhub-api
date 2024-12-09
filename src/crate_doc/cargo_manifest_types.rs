use crate::types::CrateId;
use anyhow::Result;
use cargo_manifest::Manifest;
use semver::Version;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

/// A Cargo.toml with expected metadata related to bevy
pub type CargoManifest = Manifest<ManifestMetadata>;

/// Converts bytes to string then toml
pub fn toml_from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
	let str = String::from_utf8(bytes.to_vec())?;
	let val = toml::from_str(&str)?;
	Ok(val)
}
/// Raw metadata declaration in `Cargo.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
	#[serde(default)]
	pub scene: Vec<ManifestScene>,
}

/// Cargo.toml metadata fields we can use
impl ManifestMetadata {
	/// find a scene by name
	pub fn find_scene(&self, name: &str) -> Result<&ManifestScene> {
		self.scene.iter().find(|p| p.name == name).ok_or_else(|| {
			anyhow::anyhow!("scene missing from Cargo.toml: {}", name)
		})
	}
}

/// Raw scene declaration in `Cargo.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ManifestScene {
	pub name: String,
	/// Defaults to crate description then none
	pub description: Option<String>,
	/// A url to be used for the thumbnail
	pub thumb_url: Option<String>,
	/// Some text to be used for the thumbnail
	pub thumb_text: Option<String>,
	/// Specify the location of the scene file
	/// If not specified, defaults to "scenes/{name}.json"
	pub path: Option<String>,
	/// Specify an inline scene, in json format
	pub scene_json: Option<String>,
	/// Scenes that should be included alongisde this one
	#[serde(default)]
	pub include: Vec<ManifestDependency>,
	/// Crate versions are looked up later when building the [SceneLock]
	/// Specify the
	pub app: Option<ManifestApp>,
	/// Specify details for replication events
	pub replication: Option<ManifestReplicationConfig>,
}

impl ManifestScene {
	/// Get includes, as well as optionally the scene from the app
	pub fn get_includes(&self) -> Vec<ManifestDependency> {
		let mut includes = Vec::new();
		match &self.app {
			Some(ManifestApp::Implicit(name)) => {
				includes.push(ManifestDependency::Implicit(name.clone()));
			}
			Some(ManifestApp::Explicit {
				crate_name,
				scene_name,
				..
			}) => {
				includes.push(ManifestDependency::Explicit {
					crate_name: crate_name.clone(),
					scene_name: scene_name.clone(),
				});
			}
			Some(ManifestApp::Wasm { .. }) => {}
			None => {}
		}
		includes.extend(self.include.clone());
		includes
	}
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ManifestReplicationConfig {
	pub send_events: Vec<String>,
	pub recv_events: Vec<String>,
}

/// Raw dependency declaration in `Cargo.toml`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ManifestDependency {
	/// Could be an internal scene "some-scene"
	/// Or an external one "some-crate/some-scene"
	Implicit(String),
	Explicit {
		crate_name: String,
		scene_name: String,
	},
}

fn split_implicit_path(
	default_crate_name: &str,
	value: &str,
) -> Result<(String, String)> {
	let parts = value.split('/').collect::<Vec<_>>();
	match parts.len() {
		1 => Ok((default_crate_name.to_string(), parts[0].into())),
		2 => Ok((parts[0].into(), parts[1].into())),
		_ => {
			anyhow::bail!(
				"Invalid dependency, too many parts. Please only specify internal 'my-dep' or external 'some-crate/some-dep': {}",
				value
			);
		}
	}
}

impl ManifestDependency {
	pub fn into_crate_and_scene(
		&self,
		default_crate_name: &str,
	) -> Result<(String, String)> {
		match self {
			ManifestDependency::Implicit(value) => {
				Ok(split_implicit_path(default_crate_name, value)?)
			}
			ManifestDependency::Explicit {
				crate_name,
				scene_name,
			} => Ok((crate_name.clone(), scene_name.clone())),
		}
	}
}


/// Raw app declaration in `Cargo.toml`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "kebab-case")]
pub enum ManifestApp {
	/// Could be the app of an internal scene "some-scene"
	/// Or an external one "some-crate/some-app"
	Implicit(String),
	Explicit {
		crate_name: String,
		scene_name: String,
	},
	Wasm {
		#[serde(rename = "js-url")]
		js_url: String,
		#[serde(rename = "wasm-url")]
		wasm_url: String,
		#[serde(rename = "replication-registry-url")]
		replication_registry_url: Option<String>,
		#[serde(rename = "type-registry-url")]
		type_registry_url: Option<String>,
		canvas_id: Option<String>,
	},
}

impl ManifestApp {
	pub fn replication_registry_url(&self) -> Option<String> {
		match self {
			ManifestApp::Implicit(_) => None,
			ManifestApp::Explicit { .. } => None,
			ManifestApp::Wasm {
				replication_registry_url,
				..
			} => replication_registry_url.clone(),
		}
	}
	pub fn type_registry_url(&self) -> Option<String> {
		match self {
			ManifestApp::Implicit(_) => None,
			ManifestApp::Explicit { .. } => None,
			ManifestApp::Wasm {
				type_registry_url, ..
			} => type_registry_url.clone(),
		}
	}
	pub fn canvas_id(&self) -> Option<String> {
		match self {
			ManifestApp::Implicit(_) => None,
			ManifestApp::Explicit { .. } => None,
			ManifestApp::Wasm { canvas_id, .. } => canvas_id.clone(),
		}
	}
	pub fn into_crate_and_scene(
		&self,
		default_crate_name: &str,
	) -> Result<(String, String)> {
		match self {
			ManifestApp::Implicit(value) => {
				Ok(split_implicit_path(default_crate_name, value)?)
			}
			ManifestApp::Explicit {
				crate_name,
				scene_name,
				..
			} => Ok((crate_name.clone(), scene_name.clone())),
			ManifestApp::Wasm { .. } => {
				anyhow::bail!("Wasm apps do not specify a crate and scene")
			}
		}
	}
}

/// Definition of a `Cargo.lock` file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoLock {
	version: usize,
	package: Vec<CargoLockPackage>,
}

/// Definition of a `Cargo.lock` dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoLockPackage {
	/// local packages dont have this
	checksum: Option<String>,
	/// local packages dont have this
	source: Option<String>,
	dependencies: Option<Vec<String>>,
	name: String,
	version: String,
}


impl CargoLock {
	/// Get a version from a crate
	pub fn crate_id(&self, name: &str) -> Result<CrateId> {
		let pkg = self
			.package
			.iter()
			.find(|dep| dep.name == name)
			.ok_or_else(|| {
				anyhow::anyhow!("missing dependency in Cargo.lock: {}", name)
			})?;
		let version = Version::parse(&pkg.version)?;
		Ok(CrateId::new(name, version))
	}
}

// #[cfg(test)]
#[extend::ext]
pub impl CargoManifest {
	fn bevyhub_crate_version() -> Version {
		let file = include_str!("../../../bevyhub/Cargo.toml");
		let version = toml::from_str::<CargoManifest>(&file)
			.unwrap()
			.workspace
			.unwrap()
			.package
			.unwrap()
			.version
			.unwrap();
		Version::parse(&version).unwrap()
	}
	fn bevyhub_template() -> Self {
		let file =
			include_str!("../../../bevyhub/crates/bevyhub_template/Cargo.toml");
		toml::from_str::<CargoManifest>(&file).unwrap()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use cargo_manifest_types::CargoManifestExt;
	use sweet::*;

	#[test]
	fn works() -> Result<()> {
		let manifest = CargoManifest::bevyhub_template();
		let metadata = manifest.package.unwrap().metadata.unwrap();

		let hello_world = &metadata.scene[0];
		let my_base_scene = &metadata.scene[1];
		let my_beautiful_scene = &metadata.scene[2];

		expect(&hello_world.name).to_be(&"hello-world".to_string())?;
		// expect(&hello_world.scene.unwrap().dependencies.len()).to_be(&0)?;
		expect(my_base_scene.description.clone()).to_be_some()?;
		expect(&my_beautiful_scene.include[0])
			.to_be(&ManifestDependency::Implicit("hello-world".into()))?;
		expect(&my_beautiful_scene.get_includes().len()).to_be(&2)?;
		expect(&my_beautiful_scene.get_includes()[0])
			.to_be(&ManifestDependency::Implicit("my-base-scene".into()))?;

		Ok(())
	}

	#[test]
	fn app_manifest() -> Result<()> {
		expect(
			&ManifestApp::Implicit("bar".into()).into_crate_and_scene("foo")?,
		)
		.to_be(&("foo".into(), "bar".into()))?;
		expect(
			&ManifestApp::Implicit("foo/bar".into())
				.into_crate_and_scene("bazz")?,
		)
		.to_be(&("foo".into(), "bar".into()))?;
		expect(
			ManifestApp::Implicit("foo/bar/bazz".into())
				.into_crate_and_scene("bazz"),
		)
		.to_be_err()?;

		Ok(())
	}
}
