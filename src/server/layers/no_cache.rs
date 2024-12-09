use crate::types::AppResult;
use axum::extract::Request;
use axum::http::header;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;


pub async fn no_cache(request: Request, next: Next) -> Response {
	let response = next.run(request).await;
	append_no_cache_headers(response)
}


pub fn append_no_cache_headers(val: impl IntoResponse) -> Response {
	let mut response = val.into_response();
	let headers = response.headers_mut();
	headers.insert(
		header::CACHE_CONTROL,
		HeaderValue::from_static("no-cache, no-store, must-revalidate"),
	);
	headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
	headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
	// do something with `response`...

	response
}


pub fn maybe_no_cache(val: impl IntoResponse, no_cache: bool) -> Response {
	if no_cache {
		append_no_cache_headers(val)
	} else {
		val.into_response()
	}
}

pub fn no_cache_if_latest(
	val: impl IntoResponse,
	version: &str,
) -> AppResult<Response> {
	Ok(maybe_no_cache(val, version == "latest"))
}
