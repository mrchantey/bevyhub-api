use crate::prelude::*;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::Object;
use aws_sdk_s3::Client;
use axum::body::Bytes;
use std::time::Duration;

/// S3 flavored object storage
#[derive(Clone)]
pub struct S3Storage {
	client: Client,
	bucket: String,
}


impl S3Storage {
	/// Initialize with default region
	/// In dev mode will hit a dev mode bucket
	/// https://us-west-2.console.aws.amazon.com/s3/home?region=us-west-2#
	pub async fn new(env: ApiEnvironment) -> Self {
		let shared_config = aws_config::defaults(BehaviorVersion::latest())
			.region(Region::new("us-west-2"))
			.load()
			.await;
		let client = Client::new(&shared_config);

		let bucket = match env {
			ApiEnvironment::Local => unimplemented!("local env not supported"),
			ApiEnvironment::Staging => "bevyhub-dev",
			ApiEnvironment::Prod => "bevyhub-prod",
		}
		.to_string();
		Self { client, bucket }
	}

	pub async fn list_buckets(&self) -> Result<Vec<String>> {
		let buckets = self.client.list_buckets().send().await?;
		let names = buckets
			.buckets()
			.iter()
			.map(|b| b.name().unwrap_or("unknown bucket").to_string())
			.collect();
		Ok(names)
	}
	pub async fn purge(&self) -> Result<()> {
		let objects = self
			.client
			.list_objects_v2()
			.bucket(&self.bucket)
			.send()
			.await?;
		let keys = objects
			.contents()
			.iter()
			.filter_map(|o| o.key())
			.map(|key| key.to_string())
			.collect::<Vec<String>>();
		for key in keys {
			self.client
				.delete_object()
				.bucket(&self.bucket)
				.key(&key)
				.send()
				.await?;
		}
		Ok(())
	}
}

impl Into<StorageObjectInfo> for &Object {
	fn into(self) -> StorageObjectInfo {
		let epoch_secs: u64 = self
			.last_modified() // last modified means created
			.map(|d| d.secs().try_into().unwrap_or_default())
			.unwrap_or_default();

		StorageObjectInfo {
			name: self.key().unwrap_or("unknown key").to_string(),
			created: Duration::from_secs(epoch_secs),
		}
	}
}

#[async_trait::async_trait]
impl ObjectStorage for S3Storage {
	async fn get(&self, key: &str) -> Result<Bytes> {
		let obj = self
			.client
			.get_object()
			.bucket(&self.bucket)
			.key(key)
			.send()
			.await?;
		let bytes = obj.body.collect().await.map(|data| data.into_bytes())?;
		Ok(bytes)
	}

	async fn put(&self, key: &str, value: Bytes) -> Result<()> {
		self.client
			.put_object()
			.bucket(&self.bucket)
			.key(key)
			.body(ByteStream::from(value))
			.send()
			.await?;
		Ok(())
	}

	async fn delete(&self, key: &str) -> Result<()> {
		self.client
			.delete_object()
			.bucket(&self.bucket)
			.key(key)
			.send()
			.await?;
		Ok(())
	}
	async fn list(&self, prefix: &str) -> Result<Vec<StorageObjectInfo>> {
		let objects = self
			.client
			.list_objects_v2()
			.bucket(&self.bucket)
			.prefix(prefix)
			// .max_keys(100)
			.send()
			.await?;
		let names = objects.contents().iter().map(|o| o.into()).collect();
		Ok(names)
	}


	async fn exists(&self, key: &str) -> Result<bool> {
		match self
			.client
			.head_object()
			.bucket(&self.bucket)
			.key(key)
			.send()
			.await
		{
			Ok(_) => Ok(true),
			Err(err) => {
				if let SdkError::ServiceError(err) = &err {
					if err.err().is_not_found() {
						return Ok(false);
					}
				}
				anyhow::bail!("{:?}", err)
			}
		}
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use axum::body::Bytes;
	use sweet::*;

	#[tokio::test]
	#[ignore = "hits aws"]
	async fn works() -> Result<()> {
		let storage = S3Storage::new(Default::default()).await;
		let key = "foo.txt";
		let value = Bytes::from("bar");
		storage.delete(key).await.ok();
		expect(storage.exists(key).await?).to_be_false()?;

		storage.put(key, value.clone()).await?;

		let list = storage.list("fo").await?;
		expect(&list).any(|v| v.name == key.to_string())?;
		let list = storage.list("bar").await?;
		expect(&list).not().any(|v| v.name == key.to_string())?;

		let bytes = storage.get(key).await?;
		expect(bytes).to_be(value)?;

		expect(storage.exists(key).await?).to_be_true()?;
		storage.delete(key).await.ok();
		expect(storage.exists(key).await?).to_be_false()?;

		Ok(())
	}
}
