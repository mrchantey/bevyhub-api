use anyhow::Result;
use mongodb::bson::doc;
use mongodb::options::ClientOptions;
use mongodb::options::ServerApi;
use mongodb::options::ServerApiVersion;
use mongodb::Client;
#[tokio::main]
async fn main() -> Result<()> {
	let mongodb_client = std::env::var("MONGODB_CLIENT")?;
	let mut client_options = ClientOptions::parse(mongodb_client).await?;
	// Set the server_api field of the client_options object to set the version of the Stable API on the client
	let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
	client_options.server_api = Some(server_api);
	// Get a handle to the cluster
	let client = Client::with_options(client_options)?;
	// Ping the server to see if you can connect to the cluster
	client
		.database("admin")
		.run_command(doc! {"ping": 1})
		.await?;
	println!("Pinged your deployment. You successfully connected to MongoDB!");
	Ok(())
}
