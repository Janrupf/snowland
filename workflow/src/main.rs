use std::path::Path;
use std::process::Command;

use cargo_metadata::MetadataCommand;

use crate::fail_build::{fail_build, UnwrapOrFailBuild};
use crate::packager::{Packager, PackagerError};

mod fail_build;
mod packager;

macro_rules! workflow_env {
    ($value:literal) => {
        env!(concat!("SNOWLAND_WORKFLOW_", $value))
    };
}

const CARGO_EXE: &str = workflow_env!("CARGO");
const CARGO_MANIFEST_DIR: &str = workflow_env!("CARGO_MANIFEST_DIR");

#[cfg(snowland_workflow_win_build)]
const BUILD_TARGETS: &[&str] = &["snowland-win-host", "snowland-control-panel"];

#[cfg(snowland_workflow_linux_build)]
const BUILD_TARGETS: &[&str] = &["snowland-linux-host", "snowland-control-panel"];

fn main() {
    println!("-- Building snowland release packages...");

    let metadata = MetadataCommand::new()
        .cargo_path(CARGO_EXE)
        .manifest_path(format!("{}/Cargo.toml", CARGO_MANIFEST_DIR))
        .exec()
        .unwrap_or_fail_build();

    let workspace_root = metadata.workspace_root.to_string();
    println!("-- Workspace root is {}", workspace_root);

    for target in BUILD_TARGETS {
        println!("-- Building {}", target);

        let mut args = vec!["build", "--release"];
        args.push("-p");
        args.push(target);

        let cargo_build_status = Command::new(CARGO_EXE)
            .args(&args)
            .current_dir(&workspace_root)
            .status()
            .unwrap_or_fail_build();

        if !cargo_build_status.success() {
            match cargo_build_status.code() {
                Some(i) => fail_build(format!("Cargo exited with {}", i)),
                None => fail_build("Cargo was killed by a signal"),
            }
        }
    }

    let target_dir = metadata
        .target_directory
        .into_std_path_buf()
        .join("release");

    let package_name =
        std::env::var("SNOWLAND_PACKAGE_NAME").unwrap_or_else(|_| "package.zip".into());

    let mut packager = Packager::new(&target_dir, &package_name).unwrap_or_fail_build();

    println!("-- Starting packaging, this may take some time...");
    do_package(&target_dir, &mut packager).unwrap_or_fail_build();
    do_package_os_specific(&target_dir, &mut packager).unwrap_or_fail_build();
    packager.finish();
    println!("-- Packaging finished!");
}

fn do_package(target_dir: &Path, packager: &mut Packager) -> Result<(), PackagerError> {
    packager.collect_dir(target_dir.join("data"))?;

    Ok(())
}

#[cfg(snowland_workflow_win_build)]
fn do_package_os_specific(target_dir: &Path, packager: &mut Packager) -> Result<(), PackagerError> {
    packager.collect_file(target_dir.join("snowland-control-panel.exe"))?;
    packager.collect_file(target_dir.join("snowland-win-host.exe"))?;

    Ok(())
}

#[cfg(snowland_workflow_linux_build)]
fn do_package_os_specific(target_dir: &Path, packager: &mut Packager) -> Result<(), PackagerError> {
    packager.collect_dir(target_dir.join("lib"))?;
    packager.collect_file(target_dir.join("snowland-control-panel"))?;
    packager.collect_file(target_dir.join("snowland-linux-host"))?;

    Ok(())
}
