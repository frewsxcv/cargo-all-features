use crate::{
    toolchain::{CommandTarget, RustUpToolchain},
    types::FeatureList,
    Errors, Options, Outcome,
};
use clap::ArgEnum;
use itertools::Itertools;
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};
use yansi::Paint;

pub struct Runner<'a> {
    cargo_command: &'a CargoCommand,
    command: Command,
    crate_name: &'a str,
    /// A comma separated list of features
    features: String,
    working_dir: &'a Path,
    options: Option<&'a Options>,
}

impl<'a> Runner<'a> {
    pub fn new(
        cargo_command: &'a CargoCommand,
        crate_name: &'a str,
        feature_set: &'a FeatureList<&'a String>,
        working_dir: &'a Path,
        arguments: &[String],
        options: Option<&'a Options>,
        command_target: &CommandTarget,
    ) -> Self {
        let features = feature_set.iter().join(",");

        // Building command `cargo <command> --no-default-features --features <features> <..arguments>`
        let mut command = Command::new(RustUpToolchain::cargo_cmd(command_target));
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

        if let Some(options) = options {
            if options.no_color {
                command.args(&["--color=never"]);
            }

            if options.verbose {
                println!(
                    "    {} {}",
                    Paint::blue("Running").bold(),
                    format!("{:?}", command).replace('\"', "")
                );
            }
        }

        Runner {
            crate_name,
            command,
            features,
            working_dir,
            cargo_command,
            options,
        }
    }

    pub fn run(&mut self) -> Result<Outcome, Errors> {
        println!(
            "{} crate={} features=[{}]",
            Paint::cyan(self.cargo_command.as_label()).bold(),
            self.crate_name,
            self.features
        );

        if let Some(options) = self.options {
            if options.dry_run {
                return Ok(Outcome::Pass);
            }
        }

        // Running command in work directory
        let output = self
            .command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(self.working_dir)
            .output()?;

        Ok(if output.status.success() {
            Outcome::Pass
        } else {
            Outcome::Fail(output.status)
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
    Nextest,
    Clippy,
}

impl CargoCommand {
    // Command arguments to add to cargo, `cargo <..to_cargo_arguments> ..`
    pub fn to_cargo_arguments(&self) -> &[&'static str] {
        match self {
            Self::Build => &["build"],
            Self::Check => &["check"],
            Self::Test => &["test"],
            Self::Bench => &["bench"],
            Self::MiriTest => &["miri", "test"],
            Self::Udeps => &["udeps"],
            Self::Tarpaulin => &["tarpaulin"],
            Self::Nextest => &["nextest", "run"],
            Self::Clippy => &["clippy"],
        }
    }

    // Label shown in stdout
    fn as_label(&self) -> &'static str {
        match self {
            Self::Build => "    Building ",
            Self::Check => "    Checking ",
            Self::Test => "     Testing ",
            Self::Bench => "     Benching ",
            Self::MiriTest => "     Testing with Miri ",
            Self::Udeps => "     Analyzing with Udeps ",
            Self::Tarpaulin => "     Testing with Tarpaulin ",
            Self::Nextest => "     Testing with NexTest ",
            Self::Clippy => "    Checking styling",
        }
    }
}
