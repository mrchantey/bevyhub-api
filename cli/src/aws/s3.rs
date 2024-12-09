use clap::Arg;
use clap::ArgAction;
use clap::Command;
use forky::prelude::Subcommand;

pub struct S3Command;

impl Subcommand for S3Command {
	fn name(&self) -> &'static str { "s3" }
	fn about(&self) -> &'static str { "AWS S3 commands" }
	fn append_command(&self, command: Command) -> Command {
		command.arg(
			Arg::new("ignore")
				.help("paths to ignore")
				.required(false)
				.short('i')
				.long("ignore")
				.action(ArgAction::Append),
		)
	}
}


// pub struct S3PurgeCommand;

// impl Subcommand for S3PurgeCommand {
// 	fn name(&self) -> &'static str { "purge" }
// 	fn about(&self) -> &'static str { "Purge all objects in the bucket" }
// 	fn append_command(&self, command: Command) -> Command {
// 		command.arg(
// 			Arg::new("prod")
// 				.help("hit the prod bucket")
// 				.required(false)
// 				.long("prod")
// 				.action(ArgAction::Append),
// 		)
// 	}
// 	fn run(&self, args: &clap::ArgMatches) -> Result<()> {
// 		let prod = args.get_flag("prod");
// 		tokio::runtime::Runtime::new()?.block_on(async {
// 			let s3 = S3Storage::init_with_config(prod).await;
// 			s3.purge().await?;
// 			Ok(())
// 		});
// 		Ok(())
// 	}
// }
