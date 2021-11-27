use crate::types::FeatureList;
use std::{env, error, path, process};
use strum_macros::AsRefStr;
use strum_macros::EnumString;
use termcolor::WriteColor;

pub struct TestRunner {
    command: process::Command,
    crate_name: String,
    /// A comma separated list of features
    features: String,
    working_dir: path::PathBuf,
    cargo_command: CargoCommand,
}

impl TestRunner {
    pub fn new(
        cargo_command: CargoCommand,
        crate_name: String,
        feature_set: FeatureList,
        working_dir: path::PathBuf,
    ) -> Self {
        let mut command = process::Command::new(&crate::cargo_cmd());

        command.arg(cargo_command.as_ref());
        command.arg("--no-default-features");

        let mut features = feature_set
            .iter()
            .fold(String::new(), |s, feature| s + feature + ",");

        if !features.is_empty() {
            features.remove(features.len() - 1);

            command.arg("--features");
            command.arg(&features);
        }

        // Pass through cargo args
        for arg in env::args().skip(2) {
            command.arg(&arg);
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
        match self.cargo_command {
            CargoCommand::Build => print!("    Building "),
            CargoCommand::Check => print!("    Checking "),
            CargoCommand::Test => print!("     Testing "),
        }
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

#[derive(Copy, Clone, AsRefStr, EnumString)]
pub enum CargoCommand {
    #[strum(serialize = "build")]
    Build,
    #[strum(serialize = "check")]
    Check,
    #[strum(serialize = "test")]
    Test,
}
