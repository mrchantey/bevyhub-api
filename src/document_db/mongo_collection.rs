use super::doc_id::DocId;
use crate::prelude::*;
use anyhow::Result;
use mongodb::bson::doc;
use mongodb::bson::Document;

#[async_trait::async_trait]
impl<T: HasDocId> DocumentCollection<T> for mongodb::Collection<T> {
	fn name(&self) -> &str { mongodb::Collection::<T>::name(self) }
	async fn get(&self, id: &DocId) -> Result<Option<T>> {
		let filter = doc! { "_id": id };
		let doc = mongodb::Collection::<T>::find_one(self, filter).await?;
		Ok(doc)
	}
	fn find(&self) -> FindBuilder<T> { FindBuilder::new(self) }
	async fn send_find(
		&self,
		document: Document,
		skip: Option<u64>,
		limit: Option<i64>,
	) -> Result<DocumentStream<T>> {
		let mut stream = mongodb::Collection::<T>::find(self, document);
		if let Some(limit) = limit {
			stream = stream.limit(limit);
		}
		if let Some(skip) = skip {
			stream = stream.skip(skip);
		}
		Ok(stream.await?.into())
	}

	async fn count(&self, filter: Document) -> Result<u64> {
		let count = self.count_documents(filter).await?;
		Ok(count)
	}

	async fn has(&self, id: &DocId) -> Result<bool> {
		let filter = id.to_document();
		Ok(self.count_documents(filter).await? > 0)
	}

	async fn insert(&self, doc: &T) -> Result<DocId> {
		let id = doc.doc_id();
		let query = id.to_document();
		let parsed = mongodb::bson::to_bson(doc)?;
		let doc = doc! { "$set": parsed };
		let _val = self.update_one(query, doc).upsert(true).await?;
		// let id = val.upserted_id.to_string();
		// mongodb::Collection::<T>::insert_one(self, doc)?;
		Ok(id)
	}
	async fn insert_many(&self, docs: &Vec<T>) -> Result<Vec<DocId>> {
		let futs = docs.iter().map(|doc| self.insert(doc));
		let ids = futures::future::try_join_all(futs).await?;

		// let result = mongodb::Collection::update_many(self, docs).await?;
		// let mut ids = Vec::with_capacity(result.inserted_ids.len());
		// for (i, id) in result.inserted_ids.iter() {
		// 	ids[*i] = id.clone().into();
		// }
		Ok(ids)
	}

	async fn remove(&self, id: &DocId) -> Result<bool> {
		let filter = id.to_document();
		let result = mongodb::Collection::<T>::delete_one(self, filter).await?;
		Ok(result.deleted_count == 1)
	}

	async fn clear(&self) -> Result<()> {
		mongodb::Collection::<T>::delete_many(self, doc! {}).await?;
		Ok(())
	}
}
