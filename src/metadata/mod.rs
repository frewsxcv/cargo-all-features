use crate::toolchain::{CommandTarget, RustUpToolchain};
use crate::Errors;
use std::process::{Command, Stdio};

pub mod config;
pub mod dependency;
pub mod meta_data;
pub mod package;

pub(crate) use config::*;
pub(crate) use dependency::*;
pub(crate) use meta_data::*;
pub(crate) use package::*;

/// Top element for internal handling
pub struct MetaTree {
    meta_data: MetaData,
}

impl MetaTree {
    // Parses the output of `cargo metadata` and validates the config of `cargo-all-features`
    pub fn new() -> Result<Self, Errors> {
        // Use cargo for metadata resolution
        let mut command = Command::new(RustUpToolchain::cargo_cmd(&CommandTarget::Cargo));

        command.args(&["metadata", "--format-version", "1"]);

        let output = command.stderr(Stdio::inherit()).output()?;

        if !output.status.success() {
            return Errors::CargoMetaDataNonZeroStatus {
                status: output.status,
            }
            .into();
        }

        let meta_data: MetaData = serde_json::from_slice(&output.stdout)?;

        for package in &meta_data.packages {
            package.validate()?
        }

        Ok(Self { meta_data })
    }

    // Returns a reference of `MetaData`
    pub fn meta_data(&self) -> &MetaData {
        &self.meta_data
    }
}
