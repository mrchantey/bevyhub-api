use mongodb::bson::doc;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::ops::Deref;
use ts_rs::TS;



pub const DOC_ID_PLACEHOLDER: &'static str = "DOC_ID_PLACEHOLDER";

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, TS)]
// #[serde(transparent)]
// it seems to automatically be transparent in serde_json?
pub struct DocId(pub String);

impl DocId {
	pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }

	pub fn placeholder() -> Self { Self(DOC_ID_PLACEHOLDER.into()) }

	pub fn is_placeholder(&self) -> bool { self.0 == DOC_ID_PLACEHOLDER }
	pub fn to_document(&self) -> Document {
		doc! { "_id": self.0.clone() }
	}
}

impl Default for DocId {
	fn default() -> Self { Self(DOC_ID_PLACEHOLDER.into()) }
}

impl Deref for DocId {
	type Target = String;
	fn deref(&self) -> &Self::Target { &self.0 }
}


impl Into<DocId> for String {
	fn into(self) -> DocId { DocId(self) }
}
impl Into<DocId> for Bson {
	fn into(self) -> DocId { DocId(self.to_string()) }
}

impl Into<String> for DocId {
	fn into(self) -> String { self.0 }
}
impl Into<Bson> for DocId {
	fn into(self) -> Bson { Bson::String(self.into()) }
}


impl std::fmt::Display for DocId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}


pub trait HasDocId:
	'static + Send + Sync + Debug + Clone + Serialize + DeserializeOwned
// + Into<Bson>
{
	fn doc_id(&self) -> DocId;
}


impl HasDocId for Document {
	fn doc_id(&self) -> DocId {
		let mut val = self
			.get("_id")
			.expect("mongodb Documents must have an _id field")
			.to_string();
		// remove leading and trailing quotes
		val.remove(0);
		val.pop();
		DocId(val)
	}
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use mongodb::bson::doc;
	use sweet::*;

	#[test]
	fn document() -> Result<()> {
		let my_doc = doc! { "_id": "foo" };
		let id = my_doc.doc_id();
		expect(id).to_be(DocId::new("foo"))?;

		Ok(())
	}
}
