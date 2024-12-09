use anyhow::Result;
use bevyhub_api::prelude::ApiEnvironment;
use reqwest::Client;
use serde_json::Value;

const LOCAL_HOST: &str = "http://localhost:3000";
const REMOTE_HOST: &str =
	"https://nxlsmchxgcv56pwddukyuj4dvi0zhyap.lambda-url.us-west-2.on.aws";


async fn assert_status_and_json(
	url_path: &str,
	expected_status: u16,
	expected_json: Option<Value>,
) -> Result<String> {
	let host = match ApiEnvironment::get() {
		ApiEnvironment::Local => LOCAL_HOST,
		ApiEnvironment::Staging => REMOTE_HOST,
		ApiEnvironment::Prod => REMOTE_HOST,
	};

	let url = format!("{host}{}", url_path);
	let client = Client::new();
	let response = client.get(&url).send().await?;
	let status = response.status().as_u16();
	let text = response.text().await?;
	if status != expected_status {
		anyhow::bail!(
			"status code mismatch, expected: {}
			\nstatus code: {}
			\nurl: {}
			\ntext: {}",
			expected_status,
			status,
			url_path,
			text,
		);
	}

	if let Some(expected_json) = expected_json {
		let json: Value = serde_json::from_str(&text)?;
		assert_eq!(json, expected_json);
	}
	Ok(text)
}

#[tokio::main]
async fn main() -> Result<()> {
	// just {{curl-call}} crates/bevyhub_template/versions
	// just {{curl-call}} crates/bevyhub_template/versions/0.0.6-rc.1/unpkg/README.md
	// just {{curl-call}} crates/bevyhub_template/versions/0.0.6-rc.1
	// just {{curl-call}} crates/bevyhub_template
	// just {{curl-call}} crates/bevyhub_template/versions/0.0.6-rc.1/scenes/

	// assert_status_and_json(
	// 	"/crates/bevyhub_template/versions",
	// 	200,
	// 	Some(json!([
	// 		"0.0.1",
	// 		"0.0.2",
	// 		"0.0.3",
	// 		"0.0.4",
	// 		"0.0.5",
	// 		"0.0.6-rc.1",
	// 		"0.0.6-rc.2",
	// 		"0.0.6-rc.4",
	// 		"0.0.6-rc.5",
	// 	])),
	// )
	// .await?;
	assert_status_and_json(
		"/crates/bevyhub_template/versions/latest",
		200,
		None,
	)
	.await?;
	assert_status_and_json(
		"/crates/bevyhub_template/versions/0.0.1-rc.1",
		200,
		None,
	)
	.await?;
	assert_status_and_json(
		"/crates/bevyhub_template/versions/999.99.99",
		500, //TODO 404
		None,
	)
	.await?;
	assert_status_and_json(
		"/crates/bevyhub_template/versions/0.0.1-rc.1/scenes",
		200,
		None,
	)
	.await?;
	assert_status_and_json(
		"/crates/bevyhub_template/versions/latest/scenes/my-beautiful-scene",
		200,
		None,
	)
	.await?;
	assert_status_and_json(
		"/crates/bevyhub/versions/0.0.1-rc.1/unpkg/scenes/app.json",
		200,
		None,
	)
	.await?;
	// let text = assert_status_and_json("/crates/bevyhub_template/versions/0.0.6-rc.2/scenes/my-beautiful-scene", 200, None).await?;
	// let json: Value = serde_json::from_str(&text)?;
	// println!("{}", serde_json::to_string_pretty(&json)?);


	Ok(())
}
