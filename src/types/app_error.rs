//! https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

pub type AppResult<T> = Result<T, AppError>;

// Make our own error that wraps `anyhow::Error`.
pub struct AppError {
	status_code: StatusCode,
	err: String,
}

impl AppError {
	pub fn from_status_code(status_code: StatusCode) -> Self {
		Self {
			status_code,
			err: status_code
				.canonical_reason()
				.unwrap_or("Unexpected Bevyhub server error")
				.to_string(),
		}
	}

	pub fn new(status_code: StatusCode, err: impl ToString) -> Self {
		Self {
			status_code,
			err: err.to_string(),
		}
	}

	fn log_error(&self) {
		tracing::error!("{}: {}", self.status_code, self.err);
	}
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		self.log_error();
		(self.status_code, self.err).into_response()
	}
}

// impl Into<Response<Bytes>> for AppError {
// 	fn into(self) -> Response<Bytes> {
// 		self.log_error();
// 		Response::builder()
// 			.status(self.status_code)
// 			.body(Bytes::from(self.err))
// 			.unwrap()
// 	}
// }

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
	E: ToString,
{
	fn from(err: E) -> Self {
		Self::new(StatusCode::INTERNAL_SERVER_ERROR, err)
	}
}
