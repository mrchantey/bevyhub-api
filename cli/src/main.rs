#![feature(exit_status_error, async_closure)]
use anyhow::Result;
use forky::prelude::Subcommand;

mod api;
mod aws;

#[derive(Default)]
struct Cli;

impl Subcommand for Cli {
	fn name(&self) -> &'static str { "Bevyhub API" }
	fn about(&self) -> &'static str { "Welcome to the Bevyhub API CLI!" }

	fn subcommands(&self) -> Vec<Box<dyn Subcommand>> {
		vec![Box::new(aws::S3Command), Box::new(api::PopulateCommand)]
	}
}


fn main() -> Result<()> { Cli::default().run_with_cli_args() }
