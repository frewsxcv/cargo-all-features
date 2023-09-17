use crate::types::FeatureList;
use std::{error, path, process};
use termcolor::WriteColor;

pub struct TestRunner<'a> {
    command: process::Command,
    crate_name: String,
    /// A comma separated list of features
    features: String,
    working_dir: path::PathBuf,
    cargo_command: &'a str,
}

impl<'a> TestRunner<'a> {
    pub fn new<'b: 'a>(
        cargo_command: &'b str,
        crate_name: String,
        feature_flags_last: bool,
        feature_set: FeatureList,
        cargo_args: &[String],
        working_dir: path::PathBuf,
    ) -> Self {
        let mut command = process::Command::new(&crate::cargo_cmd());

        command.arg(cargo_command);

        // Put feature arguments as last only if feature_flags_last is true to retain backwards compatibility
        let features;
        if feature_flags_last {
            TestRunner::add_cargo_args(&mut command, cargo_args);
            features = TestRunner::add_features(&mut command, &feature_set);
        } else {
            features = TestRunner::add_features(&mut command, &feature_set);
            TestRunner::add_cargo_args(&mut command, cargo_args);
        }

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
        print!("    Running {} ", self.cargo_command);
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

    fn add_features(command: &mut process::Command, feature_set: &FeatureList) -> String {
        command.arg("--no-default-features");

        let mut features = feature_set
        .iter()
        .fold(String::new(), |s, feature| s + feature + ",");

        if !features.is_empty() {
            features.remove(features.len() - 1);

            command.arg("--features");
            command.arg(&features);
        }

        features
    }

    fn add_cargo_args(command: &mut process::Command, cargo_args: &[String]) {
        for arg in cargo_args {
            command.arg(arg);
        }
    }
}

#[derive(Copy, Clone)]
pub enum CargoCommand {
    Build,
    Check,
    Test,
}

impl CargoCommand {
    pub fn get_name(self) -> &'static str {
        match self {
            CargoCommand::Build => "build",
            CargoCommand::Check => "check",
            CargoCommand::Test => "test",
        }
    }
    pub fn get_cli_name(self) -> &'static str {
        match self {
            CargoCommand::Build => "build-all-features",
            CargoCommand::Check => "check-all-features",
            CargoCommand::Test => "test-all-features",
        }
    }
}
