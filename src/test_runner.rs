use std::{env, error, path, process};
use termcolor::WriteColor;

pub struct TestRunner {
    command: process::Command,
    crate_name: String,
    feature_set: Vec<String>,
    working_dir: path::PathBuf,
}

impl TestRunner {
    pub fn new(crate_name: String, feature_set: Vec<String>, working_dir: path::PathBuf) -> Self {
        let mut command = process::Command::new(&crate::cargo_cmd());

        command.arg("test");
        command.arg("--no-default-features");

        if !feature_set.is_empty() {
            command.arg("--features");
            command.arg(&feature_set.join(","));
        }

        // Pass through cargo args
        for arg in env::args().skip(2) {
            command.arg(&arg);
        }

        TestRunner {
            crate_name,
            command,
            feature_set,
            working_dir,
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
        print!("     Testing ");
        stdout.reset().unwrap();
        println!(
            "crate={} features=[{}]",
            self.crate_name,
            self.feature_set.join(", ")
        );

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
