use bevyhub_api::prelude::*;
use std::env::set_var;

/// AWS lambda entrypoint
#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
	lambda_http::tracing::init_default_subscriber();
	set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");
	let app = server().await?;
	tracing::info!("aand we're live!\nenv: {}", ApiEnvironment::default(),);
	lambda_http::run(app).await
}
