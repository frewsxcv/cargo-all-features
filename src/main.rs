use std::{env, error, ffi, process};

mod features_finder;
mod test_runner;

fn main() -> Result<(), Box<dyn error::Error>> {
    let packages = determine_packages_to_test()?;

    for package in packages {
        let outcome = test_all_features_for_package(&package)?;

        if outcome == TestOutcome::Fail {
            break;
        }
    }

    Ok(())
}

fn test_all_features_for_package(
    package: &cargo_metadata::Package,
) -> Result<TestOutcome, Box<dyn error::Error>> {
    let feature_sets = features_finder::fetch_feature_sets(package);

    for feature_set in feature_sets {
        let mut test_runner = test_runner::TestRunner::new(
            package.name.clone(),
            feature_set.clone(),
            package
                .manifest_path
                .parent()
                .expect("could not find parent of cargo manifest path")
                .to_owned(),
        );

        let outcome = test_runner.run()?;

        if outcome == TestOutcome::Fail {
            return Ok(TestOutcome::Fail);
        }
    }

    Ok(TestOutcome::Fail)
}

fn determine_packages_to_test() -> Result<Vec<cargo_metadata::Package>, Box<dyn error::Error>> {
    let current_dir = env::current_dir()?;
    let metadata = fetch_cargo_metadata()?;

    Ok(if current_dir == metadata.workspace_root {
        metadata
            .packages
            .iter()
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .cloned()
            .collect::<Vec<cargo_metadata::Package>>()
    } else {
        vec![metadata
            .packages
            .iter()
            .find(|package| package.manifest_path.parent() == Some(&current_dir))
            .expect("Could not find cargo package in metadata")
            .to_owned()]
    })
}

fn fetch_cargo_metadata() -> Result<cargo_metadata::Metadata, Box<dyn error::Error>> {
    let json = fetch_cargo_metadata_json()?;

    Ok(serde_json::from_str(&json)?)
}

fn fetch_cargo_metadata_json() -> Result<String, Box<dyn error::Error>> {
    let mut command = process::Command::new(cargo_cmd());

    command.arg("metadata").arg("--format-version").arg("1");

    let output = command.stderr(process::Stdio::inherit()).output()?;

    if !output.status.success() {
        return Err("`cargo metadata` returned a non-zero status".into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn cargo_cmd() -> ffi::OsString {
    env::var_os("CARGO").unwrap_or_else(|| ffi::OsString::from("cargo"))
}

#[derive(Eq, PartialEq)]
pub enum TestOutcome {
    Success,
    Fail,
}
