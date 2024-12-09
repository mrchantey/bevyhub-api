use crate::prelude::*;
use anyhow::Result;
use futures::TryStreamExt;

#[derive(Debug)]
pub enum DocumentStream<T> {
	Cursor(mongodb::Cursor<T>),
	Vec(Vec<T>),
}

impl<T: HasDocId> DocumentStream<T> {
	pub async fn try_next(&mut self) -> Result<Option<T>> {
		match self {
			DocumentStream::Cursor(cursor) => Ok(cursor.try_next().await?),
			DocumentStream::Vec(vec) => Ok(vec.pop()),
		}
	}
	pub async fn try_collect(self) -> Result<Vec<T>> {
		match self {
			DocumentStream::Cursor(cursor) => Ok(cursor.try_collect().await?),
			DocumentStream::Vec(vec) => Ok(vec),
		}
	}
}

impl<T> Into<DocumentStream<T>> for mongodb::Cursor<T> {
	fn into(self) -> DocumentStream<T> { DocumentStream::Cursor(self) }
}

impl<T> Into<DocumentStream<T>> for Vec<T> {
	fn into(self) -> DocumentStream<T> { DocumentStream::Vec(self) }
}
