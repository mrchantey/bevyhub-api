use crate::prelude::*;
use axum::body::Bytes;
use axum::extract::Path;
use axum::extract::State;
use axum::middleware;
use axum::response::Json;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use semver::Version;

pub fn crate_routes() -> AppRouter {
	Router::new()
		.route(
			"/crates/:crate_name/versions",
			get(get_versions).layer(middleware::from_fn(no_cache)),
		)
		.route(
			"/crates/:crate_name/versions/:version/unpkg/*path",
			get(unpkg),
		)
		.route("/crates/:crate_name/versions/:version", get(get_crate_doc))
		.route(
			"/crates/:crate_name/versions/:version/scenes",
			get(get_crate_scene_doc_list),
		)
		.route(
			"/crates/:crate_name/versions/:version/scenes/:scene_name",
			get(get_crate_scene_doc),
		)
}


/// Get a specific file, like `scenes/my-scene.json` from a crate
async fn unpkg(
	State(api): State<Services>,
	Path((crate_name, version, file_path)): Path<(String, String, String)>,
) -> AppResult<Bytes> {
	let version = Version::parse(&version)?;
	let crate_id = CrateId::new(&crate_name, version);
	let bytes = api.get_crate_file(&crate_id, &file_path).await?;

	Ok(bytes)
}

/// Get all versions of a crate
async fn get_versions(
	State(api): State<Services>,
	Path(crate_name): Path<String>,
) -> AppResult<Json<Vec<Version>>> {
	let versions = api.registry().versions(&crate_name).await?;
	Ok(Json(versions))
}

/// Get a [CrateDoc]
async fn get_crate_doc(
	State(api): State<Services>,
	Path((crate_name, version_param)): Path<(String, String)>,
) -> AppResult<Response> {
	let version = api
		.registry()
		.version_or_latest(&crate_name, &version_param)
		.await?;
	let doc = api.crate_doc(&CrateId::new(&crate_name, version)).await?;
	no_cache_if_latest(Json(doc), &version_param)
}

/// Get a [SceneDoc] list for a crate
async fn get_crate_scene_doc_list(
	State(api): State<Services>,
	Path((crate_name, version_param)): Path<(String, String)>,
) -> AppResult<Response> {
	let version = api
		.registry()
		.version_or_latest(&crate_name, &version_param)
		.await?;
	let docs = api
		.all_scene_docs(&CrateId::new(&crate_name, version))
		.await?;
	no_cache_if_latest(Json(docs), &version_param)
}

/// Get a [SceneDoc] for a crate
async fn get_crate_scene_doc(
	State(api): State<Services>,
	Path((crate_name, version_param, scene_name)): Path<(
		String,
		String,
		String,
	)>,
) -> AppResult<Response> {
	let version = api
		.registry()
		.version_or_latest(&crate_name, &version_param)
		.await?;
	let scene_id = SceneId::with_crate_name(&crate_name, version, scene_name);
	let doc = api.scene_doc(&scene_id).await?;
	no_cache_if_latest(Json(doc), &version_param)
}
