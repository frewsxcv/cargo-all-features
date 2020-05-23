use itertools::Itertools;
use std::{env, error, ffi, process};

fn main() -> Result<(), Box<dyn error::Error>> {
    let metadata = fetch_cargo_metadata()?;

    if metadata.workspace_members.len() > 1 {
        panic!("workspaces aren't supported"); // fixme
    }

    let package_id = metadata.workspace_members[0].clone();

    let package = metadata
        .packages
        .iter()
        .find(|package| package.id == package_id)
        .unwrap();

    let feature_sets = fetch_feature_sets(package);

    for feature_set in feature_sets {
        let mut command = process::Command::new(cargo_cmd());

        command.arg("test");

        for feature in &feature_set {
            command.arg("--features");
            command.arg(feature);
        }

        println!("running: cargo test features={:?}", &feature_set);

        let output = command.stderr(process::Stdio::inherit()).output().unwrap(); // fixme

        if !output.status.success() {
            panic!("todo"); // fixme
        }

        // dbg!(command);
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

fn fetch_optional_dependencies(package: &cargo_metadata::Package) -> Vec<String> {
    package
        .dependencies
        .iter()
        .filter(|dependency| dependency.optional)
        .map(|dependency| dependency.name.to_string())
        .collect()
}

fn fetch_features(package: &cargo_metadata::Package) -> Vec<String> {
    package.features.keys().cloned().collect()
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
