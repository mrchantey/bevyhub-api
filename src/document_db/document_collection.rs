use super::doc_id::DocId;
use crate::prelude::*;
use anyhow::Result;
use mongodb::bson::Document;

#[async_trait::async_trait]
pub trait DocumentCollection<T: HasDocId>: 'static + Send + Sync {
	fn name(&self) -> &str;
	async fn get(&self, id: &DocId) -> Result<Option<T>>;
	fn find(&self) -> FindBuilder<T>;
	async fn send_find(
		&self,
		document: Document,
		skip: Option<u64>,
		limit: Option<i64>,
	) -> Result<DocumentStream<T>>;
	/// count number of documents that match the filter
	async fn count(&self, document: Document) -> Result<u64>;
	async fn has(&self, id: &DocId) -> Result<bool>;
	async fn insert(&self, doc: &T) -> Result<DocId>;
	async fn insert_many(&self, docs: &Vec<T>) -> Result<Vec<DocId>>;
	async fn remove(&self, id: &DocId) -> Result<bool>;
	/// yep, empties an entire collection, be careful!
	async fn clear(&self) -> Result<()>;
}


pub struct FindBuilder<'a, T: HasDocId> {
	pub collection: &'a dyn DocumentCollection<T>,
	pub skip: Option<u64>,
	pub limit: Option<i64>,
	pub document: Document,
}
impl<'a, T: HasDocId> FindBuilder<'a, T> {
	pub fn new(collection: &'a dyn DocumentCollection<T>) -> Self {
		Self {
			collection,
			limit: None,
			skip: None,
			document: Document::new(),
		}
	}
	pub fn skip(mut self, skip: u64) -> Self {
		self.skip = Some(skip);
		self
	}
	pub fn limit(mut self, limit: i64) -> Self {
		self.limit = Some(limit);
		self
	}
	pub fn filter(mut self, filter: Document) -> Self {
		self.document.extend(filter);
		self
	}
	pub async fn send(self) -> Result<DocumentStream<T>> {
		self.collection
			.send_find(self.document, self.skip, self.limit)
			.await
	}
}
