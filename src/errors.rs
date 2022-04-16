use crate::runner::{CargoCommand, CargoCommandOrigin};
use rayon::prelude::*;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::process::ExitStatus;
use std::string::FromUtf8Error;

/// Enumeration of errors which could be returned by this crate
#[derive(Debug)]
pub enum Errors {
    ForbiddenFlag {
        flag: &'static str,
        position: usize,
    },
    ForbiddenFlags {
        flags: Vec<Errors>,
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
    InvalidChunkNumber {
        chunk: usize,
        chunks: usize,
    },
    InvalidChunkInputs {
        chunk: usize,
        chunks: usize,
        feature_sets: usize,
    },
    RustUpNotFound,
    IoError(io::Error),
    ValidationFailed{
        message: String
    },
    SerdeJsonFailedToParse(serde_json::Error),
    CargoNotAvailable,
}

/// Implementation of erorrs for `io:Error`
impl From<io::Error> for Errors {
    fn from(error: io::Error) -> Self {
        Errors::IoError(error)
    }
}

impl<T> From<Errors> for Result<T, Box<Errors>> {
    fn from(errors: Errors) -> Self {
        Err(errors.into())
    }
}

impl<T> From<Errors> for Result<T, Errors> {
    fn from(errors: Errors) -> Self {
        Err(errors)
    }
}


/// Implementation of erorrs for `serde_json::Error`
impl From<serde_json::Error> for Errors {
    fn from(error: serde_json::Error) -> Self {
        Errors::SerdeJsonFailedToParse(error)
    }
}

/// Implementation or `Error` for `Errors`
impl error::Error for Errors {}

/// Implementation of `Display` for Errors to produce human readable messages
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
            Self::ForbiddenFlags { flags } => {
                write!(f, "the flags {} may not be used as they can interfere with the flags set by `cargo-all-features`", flags.par_iter().filter_map(|e| {
                    if let Errors::ForbiddenFlag {flag, position} = e {
                        Some(format!("{} ({})", flag, position))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>().join(", "))
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
            Self::ValidationFailed {message} => {
                write!(f, "validation of data failed. {}", message)
            }
            Self::SerdeJsonFailedToParse(errors) => {
                write!(f, "failed to parse json. {}", errors)
            }
            Self::CargoMetaDataNonZeroStatus { status } => {
                write!(f, "`cargo metadata` returned a non-zero status: {}", status)
            }
            Self::CargoNotAvailable => {
                write!(
                    f,
                    "the command `cargo` could not be found, is it installed?"
                )
            }
            Self::InvalidChunkNumber { chunk, chunks } => {
                write!(f, "the input --chunks was {} and the current chunk number (--chunk) was {}. the chunk number is not allowed to be bigger than the total amount of chunks and not smaller then 1", chunks, chunk)
            }
            Self::InvalidChunkInputs {
                chunks,
                chunk,
                feature_sets,
            } => {
                write!(f, "got invalid chunking input. not more then {} chunks allowed for this crate, chunk number ({}) is not allowed to be higher then the total chunk count ({})", feature_sets, chunk, chunks)
            }
        }
    }
}
