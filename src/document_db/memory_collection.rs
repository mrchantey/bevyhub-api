use super::doc_id::DocId;
use super::document_collection::DocumentCollection;
use crate::prelude::*;
use anyhow::Result;
use mongodb::bson::to_document;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;


fn file_path(name: &str) -> PathBuf {
	format!("target/db/{}.json", name).into()
}


/// Any regular field or the $ne and $eq filters are handled.
fn is_handled_filter(name: &str) -> bool {
	!name.starts_with("$") || vec!["$ne", "$eq", "$exists"].contains(&name)
}

#[derive(Debug, Clone)]
pub struct MemoryCollection<T> {
	pub map: Arc<RwLock<HashMap<DocId, T>>>,
	pub name: String,
	pub write_to_disk: bool,
}

impl<T: HasDocId> MemoryCollection<T> {
	/// Create a temporary collection that does not load from or write to disk.
	pub fn temp() -> Self {
		let mut this = Self::new("_temp");
		this.write_to_disk = false;
		this
	}

	pub fn new(name: impl Into<String>) -> Self {
		let name = name.into();
		let hashmap = if let Some(file) =
			std::fs::read_to_string(file_path(&name)).ok()
		{
			serde_json::from_str(&file).expect("invalid json")
		} else {
			HashMap::default()
		};

		Self {
			map: Arc::new(RwLock::new(hashmap)),
			write_to_disk: true,
			name,
		}
	}
	/// A mock mongodb filter, this is a best effort and has many inconsistencies.
	/// Real testing of queries should be done with a real mongodb instance.
	pub async fn try_filter(&self, filter: &Document) -> Vec<T> {
		if filter.is_empty() {
			return self.map.read().await.values().cloned().collect();
		}

		for (key, value) in
			filter.iter().filter(|(key, _)| !is_handled_filter(key))
		{
			tracing::warn!(
				"{}:{} filter will be ignored \nMongodb filters are not supported in memory collections",key,value
			);
		}

		// let filter = to_bson(filter).unwrap();

		self.map
			.read()
			.await
			.values()
			.filter(|doc| {
				let doc = to_document(doc).unwrap();
				compare_recursive(&doc, &filter)
			})
			.cloned()
			.collect()
	}

	async fn save_to_disk(&self, map: &HashMap<DocId, T>) -> Result<()> {
		if !self.write_to_disk {
			return Ok(());
		}
		let json = serde_json::to_string(map)?;
		let path = file_path(&self.name);
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		std::fs::write(path, json)?;
		Ok(())
	}
}

fn compare_recursive(doc: &Document, filter: &Document) -> bool {
	filter.iter().all(|(key, value)| {

		let (doc, key) = match parse_key_parts(doc, key) {
			Some(val) => val,
			None => return false,
		};

		if let Some(mongo_filter) = try_get_mongo_filter(value) {
			return compare_mongo_filter(doc.get(key), mongo_filter);
		}
		match (doc.get(key), value) {
			(Some(Bson::Document(child_doc)), Bson::Document(child_filter)) => {
				compare_recursive(child_doc, child_filter)
			}
			(Some(doc_bson), filter_bson) => doc_bson == filter_bson,
			(None, _) => false,
		}
	})
}
/// Returns the *last* document and key in the chain of keys
/// ie `{"foo.bar.baz":true}` will return the document for `bar` and the key `baz`.
fn parse_key_parts<'a>(
	mut doc: &'a Document,
	key: &'a str,
) -> Option<(&'a Document, &'a str)> {
	let mut key_parts = key.split('.').collect::<Vec<_>>();
	while key_parts.len() > 1 {
		let part = key_parts.remove(0);
		if let Some(new_doc) = doc.get(part) {
			if let Bson::Document(new_doc) = new_doc {
				doc = new_doc;
			} else {
				return None;
			}
		} else {
			return None;
		}
	}
	let key = key_parts.pop().unwrap();
	Some((doc, key))
}

/// If the value is a mongodb filter, return it.
fn try_get_mongo_filter(value: &Bson) -> Option<&Document> {
	match value {
		Bson::Document(doc) => {
			if doc.keys().any(|key| key.starts_with("$")) {
				Some(doc)
			} else {
				None
			}
		}
		_ => None,
	}
}

/// Apply a mongodb filter to a value.
fn compare_mongo_filter(value: Option<&Bson>, filter: &Document) -> bool {
	// println!("doc: {:?}, key: {}, value: {:?}", doc, key, value);
	filter.iter().all(|(filter_key, filter_value)| {
		match (filter_key.as_str(), value) {
			("$eq", Some(value)) => value == filter_value,
			("$ne", Some(value)) => value != filter_value,
			("$exists", value) => {
				if filter_value
					.as_bool()
					.expect("$exists value must be a bool")
				{
					value.is_some()
				} else {
					value.is_none()
				}
			}
			_ => false,
		}
	})
}


#[async_trait::async_trait]
impl<T: HasDocId> DocumentCollection<T> for MemoryCollection<T> {
	fn name(&self) -> &str { &self.name }
	async fn get(&self, id: &DocId) -> Result<Option<T>> {
		let map = self.map.read().await;
		let doc = map.get(id);
		Ok(doc.cloned())
	}
	fn find(&self) -> FindBuilder<T> { FindBuilder::new(self) }

	async fn count(&self, document: Document) -> Result<u64> {
		let matches = self.try_filter(&document).await;
		Ok(matches.len() as u64)
	}

	async fn has(&self, id: &DocId) -> Result<bool> {
		let map = self.map.read().await;
		Ok(map.contains_key(id))
	}

	async fn send_find(
		&self,
		document: Document,
		skip: Option<u64>,
		limit: Option<i64>,
	) -> Result<DocumentStream<T>> {
		let values = self
			.try_filter(&document)
			.await
			.into_iter()
			.skip(skip.unwrap_or(0) as usize)
			.take(limit.unwrap_or(1000) as usize)
			.collect::<Vec<_>>();
		Ok(values.into())
	}

	async fn insert(&self, doc: &T) -> Result<DocId> {
		let mut map = self.map.write().await;
		map.insert(doc.doc_id(), doc.clone());
		self.save_to_disk(&*map).await?;
		Ok(doc.doc_id())
	}
	async fn insert_many(&self, docs: &Vec<T>) -> Result<Vec<DocId>> {
		let mut map = self.map.write().await;
		let ids = docs
			.iter()
			.map(|doc| {
				map.insert(doc.doc_id(), doc.clone());
				doc.doc_id()
			})
			.collect();
		self.save_to_disk(&*map).await?;
		Ok(ids)
	}

	async fn remove(&self, id: &DocId) -> Result<bool> {
		let mut map = self.map.write().await;
		let success = map.remove(id).is_some();
		self.save_to_disk(&*map).await?;
		Ok(success)
	}
	async fn clear(&self) -> Result<()> {
		let mut map = self.map.write().await;
		map.clear();
		self.save_to_disk(&*map).await?;
		Ok(())
	}
}

// static ID_INCREMENT: AtomicUsize = AtomicUsize::new(0);
// fn next_memory_id() -> DocId {
// 	let id = ID_INCREMENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
// 	DocId(format!("memory_id_{}", id))
// }


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use mongodb::bson::doc;
	use sweet::*;

	#[tokio::test]
	async fn works() -> Result<()> {
		let collection = MemoryCollection::temp();
		// expect(collection.insert(&doc! {}).await).to_be_err_str("foo")?;

		collection.insert(&doc! {"_id":"foo"}).await?;
		expect(collection.map.write().await.get(&DocId::new("foo")))
			.to_be_some()?;
		expect(collection.get(&DocId::new("foo")).await?).to_be_some()?;
		expect(collection.get(&DocId::new("bar")).await?).to_be_none()?;

		Ok(())
	}

	#[tokio::test]
	async fn filters() -> Result<()> {
		let collection = MemoryCollection::temp();
		collection
			.insert(&doc! {
				"_id":"foo",
				"name":"bob",
				"age":null,
				"address": {
					"number": 1234,
					"street":"Main st"
				}
			})
			.await?;

		expect(collection.count(doc! {"name":"bill"}).await?).to_be(0)?;
		expect(collection.count(doc! {"name":"bob"}).await?).to_be(1)?;
		expect(collection.count(doc! { "address.number": 1234 }).await?)
			.to_be(1)?;
		expect(
			collection
				.count(doc! { "address":{"number": 1234} })
				.await?,
		)
		.to_be(1)?;
		expect(
			collection
				.count(doc! {
							"address":{
								"$eq": {
									"number": 1234,
									"street":"Main st"
								}
							}
				})
				.await?,
		)
		.to_be(1)?;
		expect(collection.count(doc! {"address":{"$ne": null}}).await?)
			.to_be(1)?;
		expect(collection.count(doc! {"age":{"$eq": null}}).await?).to_be(1)?;

		Ok(())
	}
}
