#![forbid(unsafe_code)]
#![deny(clippy::all)]

use crate::runner::CargoCommandOrigin;
use constants::{COMMAND_ORIGIN_LOOKUP_MAP, FORBIDDEN_FLAGS};
use errors::Errors;
use metadata::MetaTree;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use runner::CargoCommand;
use std::process;
use toolchain::CommandTarget;
use which::which;

pub mod constants;
pub mod errors;
pub mod metadata;
pub mod runner;
pub mod toolchain;
mod types;

#[derive(Eq, PartialEq)]
pub enum Outcome {
    Pass,
    Fail(process::ExitStatus),
}

pub struct Options {
    pub chunks: Option<usize>,
    pub chunk: Option<usize>,
    pub no_color: bool,
    pub verbose: bool,
    pub dry_run: bool,
}

pub fn run(
    cargo_command: CargoCommand,
    arguments: &[String],
    options: Option<Options>,
    command_target: CommandTarget,
) -> Result<(), Box<Errors>> {
    // Checks if any of the forbidden flags were used
    let used_forbidden_flags = FORBIDDEN_FLAGS
        .par_iter()
        .filter(|forbidden_flag| arguments.contains(&forbidden_flag.to_string()))
        .map(|flag| Errors::ForbiddenFlag {
            flag,
            position: arguments.par_iter().position_first(|e| e == flag).unwrap(),
        })
        .collect::<Vec<_>>();

    if !used_forbidden_flags.is_empty() {
        return Errors::ForbiddenFlags {
            flags: used_forbidden_flags,
        }
        .into();
    }

    // Checks if chunking options are actually valid
    if let Some(ref options) = options {
        if let Some(chunk) = options.chunk {
            if let Some(chunks) = options.chunks {
                if chunk > chunks || chunk < 1 {
                    return Errors::InvalidChunkNumber { chunk, chunks }.into();
                }
            }
        }
    }

    // Checks for the origin and availability of the command
    if let Some(origin) = COMMAND_ORIGIN_LOOKUP_MAP.get(&cargo_command) {
        match origin {
            CargoCommandOrigin::FirstParty => {
                // Nothing to do here as it should be installed, just checking if cargo is installed, just in case
                if which("cargo").is_err() {
                    return Errors::CargoNotAvailable.into();
                }
            }
            CargoCommandOrigin::RustUpComponent { name, .. } => {
                // Check what components are installed for the current toolchain
                let active_toolchain = toolchain::RustUpToolchain::active_toolchain()?;

                if !active_toolchain
                    .installed_components()?
                    .contains(&name.to_owned().to_owned())
                {
                    return Errors::OptionalRustUpComponentInstalled {
                        command: cargo_command,
                        origin: *origin,
                    }
                    .into();
                }
            }
            CargoCommandOrigin::ThirdPartyCrate { name, .. } => {
                // Checks if the cargo plugin is installed
                //
                // Cargo plugins are installed as binaries, usually something like `cargo-*`
                if which(name).is_err() {
                    return Errors::ThirdPartyCrateNotInstalled {
                        command: cargo_command,
                        origin: *origin,
                    }
                    .into();
                }
            }
        }
    } else {
        // Fails if the command could not be found in the above HashMap
        return Errors::OriginOfCommandNotFound {
            command: cargo_command,
        }
        .into();
    }

    // initiates/parses current package
    let meta_tree = MetaTree::new()?;

    // Checks which packages are in current scope
    for package in meta_tree.meta_data().determine_packages_to_run_on()? {
        // Runs the command on all feature sets
        let outcome = package.run_on_all_features(
            &cargo_command,
            arguments,
            options.as_ref(),
            &command_target,
        )?;

        if let Outcome::Fail(exit_status) = outcome {
            process::exit(exit_status.code().unwrap());
        }
    }

    Ok(())
}
