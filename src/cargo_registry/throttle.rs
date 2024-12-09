use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;


pub struct Throttle {
	last_fetch: Instant,
}
impl Default for Throttle {
	fn default() -> Self {
		Self {
			last_fetch: Instant::now() - Duration::from_secs(1), // Ensure the first request can proceed immediately
		}
	}
}

impl Throttle {
	pub fn new() -> Self { Self::default() }

	pub async fn throttle(&mut self) {
		let now = Instant::now();
		if now.duration_since(self.last_fetch) < Duration::from_secs(1) {
			sleep(Duration::from_secs(1) - now.duration_since(self.last_fetch))
				.await;
		}
		self.last_fetch = Instant::now();
	}
}
