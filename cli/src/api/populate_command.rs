use super::*;
use anyhow::Result;
use bevyhub_api::prelude::*;
use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;
use forky::prelude::Subcommand;



/// Populate the db and storage
/// Note that populating Production will hit crates.io
/// meaning all packages must actually be published
pub struct PopulateCommand;


impl Subcommand for PopulateCommand {
	fn name(&self) -> &'static str { "populate" }
	fn about(&self) -> &'static str { "Populate the api with some test data" }
	fn append_command(&self, command: Command) -> Command {
		command
			.arg(
				Arg::new("paths")
					.help("number of files to create")
					.required(true)
					.action(ArgAction::Append),
			)
			.arg(
				Arg::new("force")
					.help("repackage tarballs even if they exist")
					.action(ArgAction::SetTrue)
					.short('f')
					.long("force"),
			)
	}

	fn run(&self, args: &ArgMatches) -> Result<()> {
		tokio::runtime::Runtime::new()?.block_on(async move {
			let force = args.get_flag("force");

			let crate_ids = args
				.get_many::<String>("paths")
				.unwrap_or_default()
				.map(|p| LocalCrateId::parse(p))
				.collect::<Result<Vec<_>>>()?;

			let mut num_skipped = 0;
			let mut num_packaged = 0;
			for id in crate_ids.iter() {
				if !package_locally_if_needed(id, force)? {
					num_skipped += 1;
				} else {
					num_packaged += 1;
				}
			}
			println!(
				"packaged {num_packaged} tarballs and skipped {num_skipped}"
			);

			let api = Services::init().await?;
			if api.env == ApiEnvironment::Prod {
				println!("populating production is not allowed");
				return Ok::<(), anyhow::Error>(());
			}

			println!("populating with env {:?}", api.env);

			futures::future::try_join_all(vec![
				api.db().crates().clear(),
				api.db().scenes().clear(),
			])
			.await?;

			// let storage_futs =
			// 	crate_ids.iter().map(|id| api.crate_scenes(&id.crate_id));
			// let crates = futures::future::try_join_all(storage_futs).await?;
			let mut scene_lists = Vec::new();
			for id in crate_ids {
				// we need to do it sequentially to avoid crate upload before scene upload race
				scene_lists.push(api.all_scene_docs(&id.crate_id).await?);
			}
			let num_scenes = scene_lists.iter().map(|c| c.len()).sum::<usize>();


			println!(
				"populated {} crates with {} scenes",
				scene_lists.len(),
				num_scenes
			);

			Ok::<(), anyhow::Error>(())
		})
	}
}
