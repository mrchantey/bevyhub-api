#[cfg(test)]
mod test {
	use anyhow::Result;
	use bevyhub_api::prelude::*;
	use mongodb::bson::doc;
	use semver::Version;
	use sweet::*;

	#[tokio::test]
	async fn correctly_sets_latest() -> Result<()> {
		if ApiEnvironment::get() == ApiEnvironment::Local {
			anyhow::bail!("This test is only for remote environments, try setting API_ENV=staging or API_ENV=[staging,prod]");
		}
		let services = Services::init().await?;

		services.db().clear().await?;
		expect(
			services
				.db()
				.scenes()
				.find()
				.send()
				.await?
				.try_collect()
				.await?
				.len(),
		)
		.to_be(0)?;

		let _crate1 = services
			.crate_doc(&CrateId::new(
				"bevyhub_template",
				Version::parse("0.0.6-rc.5").unwrap(),
			))
			.await?;

		expect(
			services
				.db()
				.scenes()
				.find()
				.send()
				.await?
				.try_collect()
				.await?
				.len(),
		)
		.to_be(3)?;

		let _crate1 = services
			.crate_doc(&CrateId::new(
				"bevyhub_template",
				Version::parse("0.0.6-rc.4").unwrap(),
			))
			.await?;

		expect(
			services
				.db()
				.scenes()
				.find()
				.send()
				.await?
				.try_collect()
				.await?
				.len(),
		)
		.to_be(6)?;

		expect(
			services
				.db()
				.scenes()
				.find()
				.filter(doc! {
					"is_latest":true,
				})
				.send()
				.await?
				.try_collect()
				.await?
				.len(),
		)
		.to_be(3)?;


		Ok(())
	}
}
