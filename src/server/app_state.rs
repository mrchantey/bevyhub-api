use crate::prelude::Services;
use anyhow::Result;
use axum::extract::FromRef;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;


#[derive(Clone)]
pub struct AppState {
	uptime: Uptime,
	api: Services,
}

impl AppState {
	pub async fn new() -> Result<Self> {
		Ok(Self {
			uptime: Uptime::new(),
			api: Services::init().await?,
		})
	}
}

impl FromRef<AppState> for Services {
	fn from_ref(app_state: &AppState) -> Services { app_state.api.clone() }
}
impl FromRef<AppState> for Uptime {
	fn from_ref(app_state: &AppState) -> Uptime { app_state.uptime.clone() }
}

static NUM_UPTIME_REQUESTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Uptime {
	pub start: std::time::Instant,
}
impl Uptime {
	pub fn new() -> Self {
		Self {
			start: std::time::Instant::now(),
		}
	}
	pub fn incr_requests(&self) -> usize {
		NUM_UPTIME_REQUESTS.fetch_add(1, Ordering::SeqCst) + 1
	}

	pub fn stats(&self) -> String {
		let uptime = self.start.elapsed().as_secs();
		let requests = self.incr_requests();
		format!("Uptime: {} seconds, Requests: {}", uptime, requests)
	}
}
