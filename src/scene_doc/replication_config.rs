use crate::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReplicationConfig {
	send_events: HashMap<String, Option<bool>>,
	recv_events: HashMap<String, Option<bool>>,
}
impl ReplicationConfig {
	pub fn from_manifest(
		replication: &Option<ManifestReplicationConfig>,
	) -> Self {
		let Some(ManifestReplicationConfig {
			send_events,
			recv_events,
		}) = replication
		else {
			return Self::default();
		};
		Self {
			send_events: send_events
				.iter()
				.map(|e| (e.clone(), Some(true)))
				.collect(),
			recv_events: recv_events
				.iter()
				.map(|e| (e.clone(), Some(true)))
				.collect(),
		}
	}
}
