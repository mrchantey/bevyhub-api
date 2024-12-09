use crate::prelude::*;
use anyhow::Result;
use mongodb::bson::doc;
use mongodb::options::ClientOptions;
use mongodb::options::ServerApi;
use mongodb::options::ServerApiVersion;
use mongodb::Client;
use mongodb::Collection;
use mongodb::Database;

#[derive(Debug, Clone)]
pub struct MongoDb {
	client: Client,
	database: Database,
	scenes: Collection<SceneDoc>,
	crates: Collection<CrateDoc>,
}

impl MongoDb {
	pub async fn new(env: ApiEnvironment) -> Result<Self> {
		let conn_str = std::env::var("MONGODB_CLIENT")?;
		let mut client_options = ClientOptions::parse(conn_str).await?;
		// Set the server_api field of the client_options object to set the version of the Stable API on the client
		let server_api =
			ServerApi::builder().version(ServerApiVersion::V1).build();
		client_options.server_api = Some(server_api);
		// Get a handle to the cluster
		let client = Client::with_options(client_options)?;
		client
			.database("admin")
			.run_command(doc! {"ping": 1})
			.await?;

		let database = match env {
			ApiEnvironment::Local => {
				unimplemented!("Local MongoDb not implemented")
			}
			ApiEnvironment::Staging => client.database("db_staging"),
			ApiEnvironment::Prod => client.database("db_prod"),
		};

		Ok(Self {
			client,
			scenes: database.collection("scenes"),
			crates: database.collection("crates"),
			database,
		})
	}

	pub fn client(&self) -> &Client { &self.client }
	pub fn database(&self) -> &Database { &self.database }
}

impl DocumentDb for MongoDb {
	fn scenes(&self) -> &dyn DocumentCollection<SceneDoc> { &self.scenes }
	fn crates(&self) -> &dyn DocumentCollection<CrateDoc> { &self.crates }
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use mongodb::bson::doc;
	use mongodb::bson::Document;
	use mongodb::Collection;

	#[tokio::test]
	#[ignore = "hits mongodb"]
	async fn works() -> Result<()> {
		let db = MongoDb::new(Default::default()).await?;
		let database = db.client().database("sample_mflix");
		let my_coll: Collection<Document> = database.collection("movies");
		// Find a movie based on the title value
		let my_movie = my_coll
			.find_one(doc! { "title": "The Perils of Pauline" })
			.await?;
		println!("Found a movie:\n{:#?}", my_movie);

		Ok(())
	}
}
