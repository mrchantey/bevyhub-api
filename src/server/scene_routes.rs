use crate::prelude::*;
use axum::extract::Query;
use axum::extract::State;
use axum::middleware;
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use mongodb::bson::Bson;
use serde::Deserialize;

pub fn scene_routes() -> AppRouter {
	Router::new().route(
		"/scenes",
		get(find_scenes).layer(middleware::from_fn(no_cache)),
	)
}

/// hard limit of 100 responses
/// Ignored for in-memory databases
async fn find_scenes(
	State(api): State<Services>,
	Query(ListQuery {
		limit,
		skip,
		filter,
	}): Query<ListQuery>,
) -> AppResult<Json<Vec<SceneDoc>>> {
	let mut builder = api.db().scenes().find();
	if let Some(skip) = skip {
		builder = builder.skip(skip);
	}

	let limit = limit.unwrap_or(100).min(100);
	builder = builder.limit(limit);

	if let Some(filter) = filter {
		let json = serde_json::from_str::<serde_json::Value>(&filter)?;
		let Bson::Document(doc) = mongodb::bson::to_bson(&json)? else {
			return Err(anyhow::anyhow!(
				"filter is not a json object: {}",
				filter
			)
			.into());
		};
		tracing::info!("applying filter: {:?}", doc);
		builder = builder.filter(doc);
	}
	let scenes = builder.send().await?.try_collect().await?;
	Ok(Json(scenes))
}

#[derive(Deserialize)]
pub struct ListQuery {
	pub limit: Option<i64>,
	pub skip: Option<u64>,
	#[serde(default)]
	pub filter: Option<String>,
}
