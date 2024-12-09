use crate::prelude::*;
use anyhow::Result;
use axum::body::Bytes;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;


static USER_AGENT: &str = "contact:github.com/mrchantey/bevyhub-api";

#[derive(Default, Clone)]
pub struct CratesIo {
	throttle: Arc<RwLock<Throttle>>,
}

impl CratesIo {
	pub fn new() -> Self { Self::default() }
}
#[async_trait::async_trait]
impl CargoRegistry for CratesIo {
	async fn crate_index(&self, crate_name: &str) -> Result<CrateIndex> {
		let url = crate_index_url(crate_name);
		let client = reqwest::Client::new();
		// println!("fetching versions for {}", url);
		let res = client.get(url).send().await?;
		let res = res.error_for_status()?;

		let text = res.text().await?;
		let objs = text
			.lines()
			.filter(|line| !line.is_empty())
			.map(|json_string| serde_json::from_str(json_string))
			.collect::<Result<_, _>>()?;

		Ok(objs)
	}


	// fn get(&mut self, _crate_name: &str, _version: &str) { unimplemented!() }

	// fn get_latest(&mut self, _crate_name: &str) { unimplemented!() }

	async fn tarball(&self, crate_id: &CrateId) -> Result<Bytes> {
		self.throttle.write().await.throttle().await;

		let client = Client::builder().user_agent(USER_AGENT).build()?;

		let url = format!(
			"https://crates.io/api/v1/crates/{}/download",
			crate_id.path()
		);

		let res = client.get(url).send().await?;
		let res = res.error_for_status()?;
		Ok(res.bytes().await?)
	}
}


fn crate_index_url(crate_name: &str) -> String {
	let path = {
		let lower_crate_name = crate_name.to_lowercase();
		match lower_crate_name.len() {
			1 => format!("1/{}", lower_crate_name),
			2 => format!("2/{}", lower_crate_name),
			3 => format!("3/{}/{}", &lower_crate_name[0..1], lower_crate_name),
			_ => format!(
				"{}/{}/{}",
				&lower_crate_name[0..2],
				&lower_crate_name[2..4],
				lower_crate_name
			),
		}
	};
	format!("https://index.crates.io/{}", path)
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use crates_io::crate_index_url;
	use sweet::*;

	#[test]
	fn works() -> Result<()> {
		expect(crate_index_url("bevyhub_api").as_str())
			.to_be("https://index.crates.io/be/vy/bevyhub_api")?;

		Ok(())
	}
}
