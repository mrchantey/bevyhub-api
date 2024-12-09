use bevyhub_api::prelude::*;
use std::fs;
use std::path::PathBuf;
use ts_rs::TS;





fn main() -> anyhow::Result<()> {
	let path = PathBuf::from("bindings");
	fs::remove_dir_all(&path).ok();
	fs::create_dir_all(&path).ok();
	SceneDoc::export_all_to(&path)?;
	CrateDoc::export_all_to(&path)?;
	Ok(())
}
