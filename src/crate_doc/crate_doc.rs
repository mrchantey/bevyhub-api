use crate::prelude::*;
use anyhow::Result;
use cargo_manifest::MaybeInherited;
use cargo_manifest::Package;
use cargo_manifest::StringOrBool;
use semver::Version;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// Extracted package information from a [CargoManifest]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct CrateDoc {
	_id: DocId,
	pub crate_id: CrateId,
	pub readme: String,
	pub repository: Option<String>,
	pub description: Option<String>,
	pub keywords: Vec<String>,
	pub authors: Vec<String>,
}


impl HasDocId for CrateDoc {
	fn doc_id(&self) -> DocId { self._id.clone() }
}

impl CrateDoc {
	pub fn from_package<T>(pkg: Package<T>) -> Result<Self> {
		// todo!()
		let Package {
			name,
			version,
			readme,
			description,
			keywords,
			authors,
			repository,
			..
		} = pkg;

		let version = unwrap_inherited(version, "0.0.1".into());
		let crate_id = CrateId {
			name: name.clone(),
			version: Version::parse(&version)?,
		};

		Ok(Self {
			_id: crate_id.into_doc_id(),
			crate_id,
			readme: map_readme(readme),
			description: map_inherited(description),
			repository: map_inherited(repository),
			keywords: unwrap_inherited(keywords, Vec::new()),
			authors: unwrap_inherited(authors, Vec::new()),
		})
	}
	pub fn crate_id(&self) -> &CrateId { &self.crate_id }
}

fn map_readme(readme: Option<MaybeInherited<StringOrBool>>) -> String {
	match readme {
		Some(MaybeInherited::Local(StringOrBool::String(val))) => val,
		_ => "README.md".into(),
	}
}


fn map_inherited<T>(val: Option<MaybeInherited<T>>) -> Option<T> {
	match val {
		Some(MaybeInherited::Local(val)) => Some(val),
		_ => None,
	}
}
fn unwrap_inherited<T>(val: Option<MaybeInherited<T>>, or_else: T) -> T {
	match val {
		Some(MaybeInherited::Local(val)) => val,
		_ => or_else,
	}
}
