use std::process::{Command, Stdio};

use crate::Errors;

#[derive(Debug)]
pub struct RustUpToolchain {
    pub channel: String,
    pub triplet: String,
}

impl RustUpToolchain {
    pub fn installed_components(&self) -> Result<Vec<String>, Errors> {
        if which::which("rustup").is_err() {
            return Err(Errors::RustUpNotFound);
        }

        let output = Command::new("rustup")
            .args(&[
                format!("+{}-{}", self.channel, self.triplet),
                "component".to_string(),
                "list".to_string(),
                "--installed".to_string(),
            ])
            .stderr(Stdio::inherit())
            .output()?;

        match String::from_utf8(output.stdout) {
            Ok(value) => {
                let mut components = vec![];
                for line in value.split('\n') {
                    let parts: Vec<&str> = line.split(&format!("-{}", self.triplet)).collect();
                    if !parts.is_empty() && !parts[0].trim().is_empty() {
                        components.push(parts[0].to_owned())
                    }
                }
                Ok(components)
            }
            Err(error) => Err(Errors::FailedToParseOutputOfCommand { error }),
        }
    }

    pub fn active_toolchain() -> Result<Self, Errors> {
        if which::which("rustup").is_err() {
            return Err(Errors::RustUpNotFound);
        }

        let output = Command::new("rustup")
            .args(&["show", "active-toolchain"])
            .stderr(Stdio::inherit())
            .output()?;

        match String::from_utf8(output.stdout) {
            Ok(value) => match value.split_whitespace().next() {
                Some(value) => {
                    let parts: Vec<&str> = value.split('-').collect();

                    Ok(Self {
                        channel: parts[0].to_owned(),
                        triplet: parts[1..].join("-"),
                    })
                }
                None => Err(Errors::FailedToParseActiveToolchain {
                    output: value.to_owned(),
                }),
            },
            Err(error) => Err(Errors::FailedToParseOutputOfCommand { error }),
        }
    }
}
