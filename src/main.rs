use itertools::Itertools;
use std::{env, error, ffi, process};
use termcolor::WriteColor;

fn main() -> Result<(), Box<dyn error::Error>> {
    let metadata = fetch_cargo_metadata()?;

    // if env::current_dir()? == metadata.workspace_root {
    //     panic!("in the workspace root");
    // }

    let current_dir = env::current_dir()?;

    let package = metadata
        .packages
        .iter()
        .find(|package| package.manifest_path.parent() == Some(&current_dir))
        .unwrap();

    let feature_sets = fetch_feature_sets(package);

    for feature_set in feature_sets {
        let mut cargo_test_runner = CargoTestRunner::new(feature_set.clone());

        if !feature_set.is_empty() {
            cargo_test_runner.arg("--features");
            cargo_test_runner.arg(&feature_set.join(","));
        }

        cargo_test_runner.run();
    }

    Ok(())
}

fn fetch_feature_sets(package: &cargo_metadata::Package) -> Vec<Vec<String>> {
    let mut features = vec![];
    features.append(&mut fetch_optional_dependencies(&package));
    features.append(&mut fetch_features(&package));

    let mut feature_sets = vec![];

    for n in 0..=features.len() {
        for feature_set in features.iter().combinations(n) {
            feature_sets.push(feature_set.iter().map(|n| n.to_string()).collect());
        }
    }

    feature_sets
}

struct CargoTestRunner {
    command: process::Command,
    feature_set: Vec<String>,
}

impl CargoTestRunner {
    fn new(feature_set: Vec<String>) -> Self {
        let command = process::Command::new(&cargo_cmd());

        let mut s = CargoTestRunner { command, feature_set };
        s.arg("test");
        s.arg("--no-default-features");

        s
    }

    fn arg(&mut self, arg: &str) {
        self.command.arg(arg);
    }

    fn run(&mut self) {
        let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
        stdout.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
        print!("    Features ");
        stdout.reset().unwrap();
        println!("[{}]", self.feature_set.join(", "));

        let output = self.command.stderr(process::Stdio::inherit()).output().unwrap(); // fixme

        if !output.status.success() {
            panic!("todo"); // fixme
        }

    }
}

fn fetch_optional_dependencies(package: &cargo_metadata::Package) -> Vec<String> {
    package
        .dependencies
        .iter()
        .filter(|dependency| dependency.optional)
        .map(|dependency| dependency.name.to_string())
        .collect()
}

fn fetch_features(package: &cargo_metadata::Package) -> Vec<String> {
    package.features.keys().filter(|key| key != &"default").cloned().collect()
}

fn fetch_cargo_metadata() -> Result<cargo_metadata::Metadata, Box<dyn error::Error>> {
    let json = fetch_cargo_metadata_json()?;

    Ok(serde_json::from_str(&json)?)
}

fn fetch_cargo_metadata_json() -> Result<String, Box<dyn error::Error>> {
    let mut command = process::Command::new(cargo_cmd());

    command.arg("metadata").arg("--format-version").arg("1");

    // fixme: cargo metadata

    let output = command.stderr(process::Stdio::inherit()).output().unwrap(); // fixme

    if !output.status.success() {
        panic!("todo"); // fixme
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn cargo_cmd() -> ffi::OsString {
    env::var_os("CARGO").unwrap_or_else(|| ffi::OsString::from("cargo"))
}
