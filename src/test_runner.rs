use crate::types::FeatureList;
use std::{error, path, process};
use termcolor::WriteColor;

pub struct TestRunner {
    command: process::Command,
    crate_name: String,
    /// A comma separated list of features
    features: String,
    working_dir: path::PathBuf,
    cargo_command: String,
}

fn split_slice<'a>(slice: &'a [String], item: &'a str) -> (&'a [String], &'a [String]) {
    if let Some(pos) = slice.iter().position(|s| s == item) {
        (&slice[..pos], &slice[pos..])
    } else {
        (slice, &[])
    }
}

impl TestRunner {
    pub fn new(
        cargo_command: String,
        crate_name: String,
        feature_set: FeatureList,
        cargo_args: &[String],
        working_dir: path::PathBuf,
    ) -> Self {
        let mut command = process::Command::new(crate::cargo_cmd());

        command.arg(cargo_command.clone());

        let (cargo_args_b, cargo_args_a) = split_slice(cargo_args, "--");

        // Pass through cargo args
        for arg in cargo_args_b {
            command.arg(arg);
        }

        // Pass through cargo args
        // Example: `cargo all-features clippy --no-deps -- --package xyz`
        // We take `clippy` and `--no-deps` for now
        command.args(cargo_args_b.iter());

        // We add `--no-default-features`
        command.arg("--no-default-features");

        // We add feature set `--features [any combination]`
        let mut features = feature_set
            .iter()
            .fold(String::new(), |s, feature| s + feature + ",");

        if !features.is_empty() {
            features.remove(features.len() - 1);

            command.arg("--features");
            command.arg(&features);
        }

        // And last we pass `--` and `--package xyz` to command args
        command.args(cargo_args_a.iter());

        // We successfully constructed `cargo clippy --no-deps --no-default-features --features [any combination] -- --package xyz`
        TestRunner {
            crate_name,
            command,
            features,
            working_dir,
            cargo_command,
        }
    }

    pub fn run(&mut self) -> Result<crate::TestOutcome, Box<dyn error::Error>> {
        let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
        stdout
            .set_color(
                termcolor::ColorSpec::new()
                    .set_fg(Some(termcolor::Color::Cyan))
                    .set_bold(true),
            )
            .unwrap();
        print!("     Running {} ", self.cargo_command);
        stdout.reset().unwrap();
        println!("crate={} features=[{}]", self.crate_name, self.features);

        let output = self
            .command
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .current_dir(&self.working_dir)
            .output()?;

        Ok(if output.status.success() {
            crate::TestOutcome::Pass
        } else {
            crate::TestOutcome::Fail(output.status)
        })
    }
}
