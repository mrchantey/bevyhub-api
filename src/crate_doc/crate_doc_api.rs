use crate::prelude::*;
use anyhow::Result;

impl Services {
	pub async fn crate_doc(&self, crate_id: &CrateId) -> Result<CrateDoc> {
		if let Some(crate_doc) =
			self.db().crates().get(&crate_id.into_doc_id()).await?
		{
			return Ok(crate_doc);
		} else {
			let (crate_doc, _) = self.unpack_crate_to_db(crate_id).await?;
			Ok(crate_doc)
		}
	}
}
