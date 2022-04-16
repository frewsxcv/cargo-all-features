use crate::Errors;
use clap::ArgEnum;
use rayon::prelude::*;
use std::process::{Command, Stdio};
use which::which;

#[derive(ArgEnum, Clone, Debug, PartialEq)]
pub enum CommandTarget {
    Cargo,
    Cross,
}

#[derive(Debug)]
pub struct RustUpToolchain {
    pub channel: String,
    pub triplet: String,
}

impl RustUpToolchain {
    // Get value of `CARGO` env variable at runtime or use `cargo`
    pub fn cargo_cmd(target: &CommandTarget) -> String {
        match target {
            CommandTarget::Cargo => {
                std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
            }
            CommandTarget::Cross => String::from("cross"),
        }
    }

    // List installed component of toolchain
    pub fn installed_components(&self) -> Result<Vec<String>, Errors> {
        // Checking if `rustup` is available
        if which("rustup").is_err() {
            return Errors::RustUpNotFound.into();
        }

        // Running `rustup +<toolchain> component list --installed`
        let output = Command::new("rustup")
            .args(&[
                format!("+{}-{}", self.channel, self.triplet),
                "component".to_string(),
                "list".to_string(),
                "--installed".to_string(),
            ])
            .stderr(Stdio::inherit())
            .output()?;

        // Parse output of command
        match String::from_utf8(output.stdout) {
            Ok(value) => {
                // Split at each new line and parse each line as `<component>-<toolchain>..`
                let components = value
                    .par_split('\n')
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split(&format!("-{}", self.triplet)).collect();
                        if !parts.is_empty() && !parts[0].trim().is_empty() {
                            Some(parts[0].to_owned())
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(components)
            }
            Err(error) => Errors::FailedToParseOutputOfCommand { error }.into(),
        }
    }

    // Parse RustUps active toolchain
    pub fn active_toolchain() -> Result<Self, Errors> {
        // Checking if `rustup` is available
        if which("rustup").is_err() {
            return Errors::RustUpNotFound.into();
        }

        // running `rustup show active-toolchain`
        let output = Command::new("rustup")
            .args(&["show", "active-toolchain"])
            .stderr(Stdio::inherit())
            .output()?;

        // Parse output of command
        match String::from_utf8(output.stdout) {
            // Split at whitespace as output will be `<channel_or_version>-<triplet> (<reason>)`
            Ok(value) => match value.split_whitespace().next() {
                Some(value) => {
                    // Split channel or version from triplet
                    let parts: Vec<&str> = value.par_split('-').collect();
                    println!("{:?}", parts);
                    Ok(Self {
                        channel: parts[0].to_owned(),
                        triplet: parts[1..].join("-"),
                    })
                }
                None => Errors::FailedToParseActiveToolchain {
                    output: value.to_owned(),
                }
                .into(),
            },
            Err(error) => Errors::FailedToParseOutputOfCommand { error }.into(),
        }
    }
}
