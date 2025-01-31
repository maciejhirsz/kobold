use std::env;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::report::{Error, ErrorExt, Report};

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    target_directory: PathBuf,
}

#[derive(Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub manifest_path: PathBuf,
}

pub struct Manifest {
    pub package: Package,
    pub target: PathBuf,
}

pub fn manifest(package: Option<&str>) -> Report<Manifest> {
    let working_path = env::current_dir().message("failed to read current directory")?;
    let manifest_path = working_path.join("Cargo.toml");

    let out = Command::new("cargo")
        .args([
            "metadata",
            "--format-version=1",
            "--filter-platform=wasm32-unknown-unknown",
            "--no-deps",
        ])
        .output()
        .message("failed to run cargo")?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(Error::message(format!(
            "failed to read cargo metadata\n{err}",
        )));
    }

    let metadata: Metadata =
        serde_json::from_slice(&out.stdout).message("failed to parse cargo metadata")?;

    let current = match package {
        Some(name) => metadata
            .packages
            .into_iter()
            .find(|package| package.name == name)
            .ok_or_else(|| Error::message(format!("a package with name {name} not found")))?,
        None => metadata
            .packages
            .into_iter()
            .find(|package| package.manifest_path == manifest_path)
            .ok_or_else(|| {
                Error::message(format!(
                    "a package with manifest {} not found",
                    manifest_path.display(),
                ))
            })
            .note("specify the package name with `--package <name>` or `-p <name>`")?,
    };

    Ok(Manifest {
        package: current,
        target: metadata.target_directory,
    })
}
