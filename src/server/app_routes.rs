use crate::prelude::*;
use axum::body::Body;
use axum::extract::Path;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use reqwest::header;
use reqwest::StatusCode;
use std::fs;


const APP_PATH: &str = "/home/pete/me/bevyhub-apps";


pub fn app_routes() -> AppRouter {
	Router::new().route("/apps/*path", get(get_app))
}
#[axum::debug_handler]
async fn get_app(
	// State(api): State<Services>,
	Path(file_path): Path<String>,
) -> AppResult<Response<Body>> {
	let path = format!("{APP_PATH}/{file_path}");
	let bytes = fs::read(&path)
		.map_err(|_| anyhow::anyhow!("File not found: {}", &path))?;

	let content_type = if file_path.ends_with(".wasm") {
		"application/wasm"
	} else if file_path.ends_with(".js") {
		"application/javascript"
	} else if file_path.ends_with(".css") {
		"text/css"
	} else {
		"application/octet-stream"
	};

	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, content_type)
		.body(bytes.into())?;
	Ok(response)
}
