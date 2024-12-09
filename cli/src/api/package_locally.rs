use super::*;
use anyhow::Result;
use bevyhub_api::types::CrateId;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// If `--force` or tarball is missing from local cache it will
/// be packaged and copied locally
pub fn package_locally_if_needed(
	id: &LocalCrateId,
	force: bool,
) -> Result<bool> {
	let cargo_target_dir = std::env::var("CARGO_TARGET_DIR")
		.unwrap_or_else(|_| "target".to_string());
	let pkg_src =
		tarball_path(&format!("{}/package", &cargo_target_dir), &id.crate_id);
	let pkg_dst = "target/tarball-cache";

	if !force && fs::exists(tarball_path(pkg_dst, &id.crate_id))? {
		return Ok(false);
	}

	Command::new("cargo")
		.args(&[
			"package",
			"--no-verify",
			"--allow-dirty",
			"--manifest-path",
			&id.path.join("Cargo.toml").to_string_lossy(),
		])
		.status()?
		.exit_ok()?;

	std::fs::create_dir_all(pkg_dst).ok();
	std::thread::sleep(std::time::Duration::from_secs(1));

	Command::new("cp")
		.args(&[&pkg_src, &pkg_dst.into()])
		.status()?
		.exit_ok()?;

	Ok(true)
}


fn tarball_path(prefix: &str, id: &CrateId) -> PathBuf {
	format!("{}/{}-{}.crate", prefix, id.name, id.version).into()
}
