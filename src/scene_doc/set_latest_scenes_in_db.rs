use crate::prelude::*;
use anyhow::Result;
use mongodb::bson::doc;
use mongodb::bson::Document;

#[extend::ext(name=SetLatestScenesInDbExt)]
pub impl Services {
	/// Finds all scenes that are latest and ensures `is_latest: true`.
	/// Also finds all scenes that are not latest and ensures `is_latest: false`.
	async fn set_latest_scenes_in_db(&self, crate_id: &CrateId) -> Result<()> {
		let latest_version =
			self.registry().latest_version(&crate_id.name).await?;

		// entries that:
		// 1. have the same crate name
		// 2. not the latest version
		// 3. are marked as latest
		let mut should_not_be_latest = get_scenes(self, doc! {
			"scene_id.crate_id":{
					"crate_name": &crate_id.name,
					"version":	{
						"$ne": latest_version.to_string()
					}
			},
			"is_latest": true
		})
		.await?;


		for scene in should_not_be_latest.iter_mut() {
			scene.is_latest = false;
		}

		// entries that:
		// 1. are the latest version of this crate
		// 2. are not marked as latest
		let mut should_be_latest = get_scenes(self, doc! {
			"scene_id.crate_id": CrateId::new(&crate_id.name, latest_version.clone()),
			"is_latest": false
		})
		.await?;
		// println!(
		// 	"should_not_be_latest: {}, should_be_latest: {}",
		// 	should_not_be_latest.len(),
		// 	should_be_latest.len()
		// );
		for scene in should_be_latest.iter_mut() {
			scene.is_latest = true;
		}

		let all_scenes = should_not_be_latest
			.into_iter()
			.chain(should_be_latest.into_iter())
			.collect::<Vec<_>>();

		self.db().scenes().insert_many(&all_scenes).await?;

		Ok(())
	}
}

async fn get_scenes(api: &Services, filter: Document) -> Result<Vec<SceneDoc>> {
	let scenes = api
		.db()
		.scenes()
		.find()
		.filter(filter)
		.send()
		.await?
		.try_collect()
		.await?;
	Ok(scenes)
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use mongodb::bson::doc;
	use sweet::*;

	//TODO this test is inadequate, just checks whether some happened to be latest
	#[tokio::test]
	async fn works() -> Result<()> {
		let api = Services::init().await?;


		expect(
			api.db()
				.scenes()
				.find()
				.filter(doc! {"is_latest":true })
				.send()
				.await?
				.try_collect()
				.await?
				.len(),
		)
		.to_be_greater_than(1)?;

		Ok(())
	}
}
