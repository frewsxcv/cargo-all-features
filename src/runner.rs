use crate::{cargo_cmd, types::FeatureList, Errors, TestOutcome};
use clap::ArgEnum;
use itertools::Itertools;
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Runner<'a> {
    cargo_command: &'a CargoCommand,
    command: Command,
    crate_name: &'a str,
    /// A comma separated list of features
    features: String,
    working_dir: &'a Path,
}

impl<'a> Runner<'a> {
    pub fn new(
        cargo_command: &'a CargoCommand,
        crate_name: &'a str,
        feature_set: FeatureList<&'a String>,
        working_dir: &'a Path,
        arguments: &[String],
    ) -> Self {
        let features = feature_set.iter().join(",");

        let mut command = Command::new(cargo_cmd());
        command.args(cargo_command.to_cargo_arguments());
        command.args(&["--no-default-features"]);

        if !features.is_empty() {
            command.arg("--features");
            command.arg(&features);
        }

        // Pass through cargo args
        if !arguments.is_empty() {
            command.args(arguments.iter().map(OsStr::new));
        }

        Runner {
            crate_name,
            command,
            features,
            working_dir,
            cargo_command,
        }
    }

    pub fn run(&mut self) -> Result<TestOutcome, Errors> {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))
            .unwrap();
        print!("{}", self.cargo_command.as_label());
        stdout.reset().unwrap();
        println!("crate={} features=[{}]", self.crate_name, self.features);

        let output = self
            .command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(self.working_dir)
            .output()?;

        Ok(if output.status.success() {
            TestOutcome::Pass
        } else {
            TestOutcome::Fail(output.status)
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CargoCommandOrigin {
    FirstParty,
    RustUpComponent {
        name: &'static str,
        help_url: &'static str,
    },
    ThirdPartyCrate {
        name: &'static str,
        help_url: &'static str,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, ArgEnum)]
pub enum CargoCommand {
    Build,
    Check,
    Test,
    Bench,
    MiriTest,
    Udeps,
    Tarpaulin,
}

impl CargoCommand {
    pub fn to_cargo_arguments(&self) -> &[&'static str] {
        match self {
            Self::Build => &["build"],
            Self::Check => &["check"],
            Self::Test => &["test"],
            Self::Bench => &["bench"],
            Self::MiriTest => &["miri", "test"],
            Self::Udeps => &["udeps"],
            Self::Tarpaulin => &["tarpaulin"],
        }
    }

    fn as_label(&self) -> &'static str {
        match self {
            Self::Build => "    Building ",
            Self::Check => "    Checking ",
            Self::Test => "     Testing ",
            Self::Bench => "     Benching ",
            Self::MiriTest => "     Testing with Miri ",
            Self::Udeps => "     Analyzing with Udeps ",
            Self::Tarpaulin => "     Testing with Tarpaulin ",
        }
    }
}
