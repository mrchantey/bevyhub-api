use crate::prelude::*;
use anyhow::Result;
use mongodb::bson::doc;

impl Services {
	/// Get a scene from the db, and try to populate if it doesn't exist.
	pub async fn scene_doc(&self, scene_id: &SceneId) -> Result<SceneDoc> {
		if let Some(scene) =
			self.db().scenes().get(&scene_id.into_doc_id()).await?
		{
			// scene found, happy days
			return Ok(scene);
		} else if self
			.db()
			.crates()
			.has(&scene_id.crate_id().into_doc_id())
			.await?
		{
			anyhow::bail!(
				"This crate exists but the scene {} was not found",
				scene_id
			);
		} else {
			let (_, scenes) =
				self.unpack_crate_to_db(scene_id.crate_id()).await?;
			let scene = scenes
				.into_iter()
				.find(|scene| scene.scene_id == *scene_id)
				.ok_or_else(|| {
					anyhow::anyhow!(
						"This crate exists but the scene {} was not found",
						scene_id
					)
				})?;
			Ok(scene)
		}
	}

	pub async fn all_scene_docs(
		&self,
		crate_id: &CrateId,
	) -> Result<Vec<SceneDoc>> {
		let scenes = self
			.db()
			.scenes()
			.find()
			.filter(doc! {
				"scene_id":{
					"crate_id": crate_id
				}
			})
			.send()
			.await?
			.try_collect()
			.await?;

		if scenes.len() > 0
			|| self.db().crates().has(&crate_id.into_doc_id()).await?
		{
			Ok(scenes)
		} else {
			let (_, scenes) = self.unpack_crate_to_db(crate_id).await?;
			Ok(scenes)
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
		let result = api
			.all_scene_docs(&CrateId::bevyhub_template_bad_version())
			.await;
		expect(result).to_be_err()?;
		expect(
			api.all_scene_docs(&CrateId::bevyhub_template())
				.await?
				.len(),
		)
		.to_be(3)?;

		Ok(())
	}
}
