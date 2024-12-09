use crate::prelude::*;

#[derive(Clone)]
pub struct MemoryDb {
	// collections: HashMap<String, MemoryCollection<Bytes>>,
	scenes: MemoryCollection<SceneDoc>,
	crates: MemoryCollection<CrateDoc>,
}

impl MemoryDb {
	pub fn new() -> Self {
		Self {
			// collections: HashMap::new(),
			scenes: MemoryCollection::new("scenes"),
			crates: MemoryCollection::new("crates"),
		}
	}
}

impl DocumentDb for MemoryDb {
	fn scenes(&self) -> &dyn DocumentCollection<SceneDoc> { &self.scenes }
	fn crates(&self) -> &dyn DocumentCollection<CrateDoc> { &self.crates }
}


// See memory_collection.rs for tests
