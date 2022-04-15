#![forbid(unsafe_code)]
#![deny(clippy::all)]

use core::fmt::Display;
use metadata::MetaTree;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use runner::CargoCommand;
use std::collections::HashMap;
use std::fmt::{self, Formatter};
use std::process::ExitStatus;
use std::string::FromUtf8Error;
use std::{env, error, io, process};
use validator::ValidationError;

use crate::runner::CargoCommandOrigin;

pub mod metadata;
pub mod runner;
pub mod toolchain;
mod types;

const FORBIDDEN_FLAGS: [&str; 19] = [
    "--no-default-features",
    "--features",
    "--bin",
    "--lib",
    "-p",
    "--bins",
    "--workspace",
    "--example",
    "--examples",
    "--test",
    "--tests",
    "--bench",
    "--benches",
    "--all-targets",
    "--manifest-path",
    "--all",
    "--exclude",
    "--bins",
    "--libs",
];

#[derive(Debug)]
pub enum Errors {
    ForbiddenFlag {
        flag: &'static str,
        position: usize,
    },
    OriginOfCommandNotFound {
        command: CargoCommand,
    },
    ThirdPartyCrateNotInstalled {
        command: CargoCommand,
        origin: CargoCommandOrigin,
    },
    OptionalRustUpComponentInstalled {
        command: CargoCommand,
        origin: CargoCommandOrigin,
    },
    FailedToParseActiveToolchain {
        output: String,
    },
    FailedToParseOutputOfCommand {
        error: FromUtf8Error,
    },
    CargoMetaDataNonZeroStatus {
        status: ExitStatus,
    },
    RustUpNotFound,
    IoError(io::Error),
    ValidationFailed(ValidationError),
    SerdeJsonFailedToParse(serde_json::Error),
}

impl From<io::Error> for Errors {
    fn from(error: io::Error) -> Self {
        Errors::IoError(error)
    }
}

impl From<ValidationError> for Errors {
    fn from(error: ValidationError) -> Self {
        Errors::ValidationFailed(error)
    }
}

impl From<serde_json::Error> for Errors {
    fn from(error: serde_json::Error) -> Self {
        Errors::SerdeJsonFailedToParse(error)
    }
}

impl error::Error for Errors {}

impl Display for Errors {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ForbiddenFlag { flag, .. } => {
                write!(
                    f,
                    "the flag {:?} may not be used as it can interfere with the flags set by `cargo-all-features`",
                    flag
                )
            }
            Self::OriginOfCommandNotFound { command } => {
                write!(
                    f,
                    "origin of supplied command ({:?}) could not be found, please report this issue",
                    command
                )
            }
            Self::OptionalRustUpComponentInstalled { command, origin } => {
                if let CargoCommandOrigin::RustUpComponent { name, help_url } = origin {
                    write!(
                        f,
                        "could not run `cargo {}` as the rustup component {} is not installed.y\nTo install, run `rustup component add {}` or have a look at {}",
                        command.to_cargo_arguments().join(" "),
                        name,
                        name,
                        help_url
                    )
                } else {
                    write!(f, "supplied error does not match command origin, please report this, command: {:?}, origin: {:?}", command, origin)
                }
            }
            Self::ThirdPartyCrateNotInstalled { command, origin } => {
                if let CargoCommandOrigin::ThirdPartyCrate { name, help_url } = origin {
                    write!(
                        f,
                        "could not run `cargo {}` as the cargo binary {} is not installed.y\nTo install, run `cargo install {}` or have a look at {}",
                        command.to_cargo_arguments().join(" "),
                        name,
                        name,
                        help_url
                    )
                } else {
                    write!(f, "supplied error does not match command origin, please report this, command: {:?}, origin: {:?}", command, origin)
                }
            }
            Self::FailedToParseActiveToolchain { output } => {
                write!(f, "failed to parse the output of the command `rustup show active toolchain`, its output was: {:?}", output)
            }
            Self::RustUpNotFound => {
                write!(f, "the `rustup` command seems missing, is it installed?")
            }
            Self::IoError(value) => {
                write!(f, "an io::Error ocurred, here's its output: {}", value)
            }
            Self::FailedToParseOutputOfCommand { error } => {
                write!(f, "an error ocurred during utf8 parsing. {}", error)
            }
            Self::ValidationFailed(errors) => {
                write!(f, "validation of data failed. {}", errors)
            }
            Self::SerdeJsonFailedToParse(errors) => {
                write!(f, "failed to parse json. {}", errors)
            }
            Self::CargoMetaDataNonZeroStatus { status } => {
                write!(f, "`cargo metadata` returned a non-zero status: {}", status)
            }
        }
    }
}

lazy_static::lazy_static! {
    static ref COMMAND_ORIGIN_LOOKUP_MAP: HashMap<CargoCommand, CargoCommandOrigin> = [
        (CargoCommand::Build, CargoCommandOrigin::FirstParty),
        (CargoCommand::Check, CargoCommandOrigin::FirstParty),
        (CargoCommand::Test, CargoCommandOrigin::FirstParty),
        (CargoCommand::Bench, CargoCommandOrigin::FirstParty),
        (CargoCommand::MiriTest, CargoCommandOrigin::RustUpComponent {
            name: "miri",
            help_url: "https://github.com/rust-lang/miri"
        }),
        (CargoCommand::Udeps, CargoCommandOrigin::ThirdPartyCrate {
            name: "cargo-udeps",
            help_url: "https://github.com/est31/cargo-udeps"
        }),
        (CargoCommand::Tarpaulin, CargoCommandOrigin::ThirdPartyCrate {
            name: "cargo-tarpaulin",
            help_url: "https://github.com/est31/cargo-udeps"
        }),
    ].par_iter().copied().collect();
}

pub fn run(cargo_command: CargoCommand, arguments: &[String]) -> Result<(), Box<Errors>> {
    for forbidden_flag in FORBIDDEN_FLAGS {
        if arguments.contains(&forbidden_flag.to_string()) {
            return Err(Box::new(Errors::ForbiddenFlag {
                flag: forbidden_flag,
                position: arguments
                    .par_iter()
                    .position_first(|e| e == forbidden_flag)
                    .unwrap(),
            }));
        }
    }

    if let Some(origin) = COMMAND_ORIGIN_LOOKUP_MAP.get(&cargo_command) {
        match origin {
            CargoCommandOrigin::FirstParty => {}
            CargoCommandOrigin::RustUpComponent { name, .. } => {
                let active_toolchain = toolchain::RustUpToolchain::active_toolchain()?;

                if !active_toolchain
                    .installed_components()?
                    .contains(&name.to_owned().to_owned())
                {
                    return Err(Box::new(Errors::OptionalRustUpComponentInstalled {
                        command: cargo_command,
                        origin: *origin,
                    }));
                }
            }
            CargoCommandOrigin::ThirdPartyCrate { name, .. } => {
                if which::which(name).is_err() {
                    return Err(Box::new(Errors::ThirdPartyCrateNotInstalled {
                        command: cargo_command,
                        origin: *origin,
                    }));
                }
            }
        }
    } else {
        return Err(Box::new(Errors::OriginOfCommandNotFound {
            command: cargo_command,
        }));
    }

    let meta_tree = MetaTree::new()?;

    for package in meta_tree.meta_data().determine_packages_to_run_on()? {
        let outcome = package.run_on_all_features(&cargo_command, arguments)?;

        if let TestOutcome::Fail(exit_status) = outcome {
            process::exit(exit_status.code().unwrap());
        }
    }

    Ok(())
}

fn cargo_cmd() -> String {
    env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

#[derive(Eq, PartialEq)]
pub enum TestOutcome {
    Pass,
    Fail(process::ExitStatus),
}
