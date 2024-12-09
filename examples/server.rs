use anyhow::Result;
use bevyhub_api::prelude::*;


/// Almost identical to `main.rs` but uses [`axum::serve`] instead of [`lambda_http::run`]
#[tokio::main]
async fn main() -> Result<()> {
	lambda_http::tracing::init_default_subscriber();
	let router = server().await?;
	let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
	tracing::info!(
		"listening on {}\nenv: {}",
		listener.local_addr()?,
		ApiEnvironment::default(),
	);
	axum::serve(listener, router).await?;
	Ok(())
}
