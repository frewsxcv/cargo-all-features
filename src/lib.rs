use std::{env, error, ffi, process};

pub mod cargo_metadata;
pub mod features_finder;
pub mod test_runner;
mod types;

pub fn run(cargo_command: test_runner::CargoCommand) -> Result<(), Box<dyn error::Error>> {
    let packages = determine_packages_to_test()?;

    for package in packages {
        let outcome = test_all_features_for_package(&package, cargo_command)?;

        if let TestOutcome::Fail(exit_status) = outcome {
            process::exit(exit_status.code().unwrap());
        }
    }

    Ok(())
}

fn test_all_features_for_package(
    package: &cargo_metadata::Package,
    command: crate::test_runner::CargoCommand,
) -> Result<TestOutcome, Box<dyn error::Error>> {
    let feature_sets = crate::features_finder::fetch_feature_sets(package);

    for feature_set in feature_sets {
        let mut test_runner = crate::test_runner::TestRunner::new(
            command,
            package.name.clone(),
            feature_set.clone(),
            package
                .manifest_path
                .parent()
                .expect("could not find parent of cargo manifest path")
                .to_owned(),
        );

        let outcome = test_runner.run()?;

        match outcome {
            TestOutcome::Pass => (),
            // Fail fast if we encounter a test failure
            t @ TestOutcome::Fail(_) => return Ok(t),
        }
    }

    Ok(TestOutcome::Pass)
}

fn determine_packages_to_test() -> Result<Vec<cargo_metadata::Package>, Box<dyn error::Error>> {
    let current_dir = env::current_dir()?;
    let metadata = cargo_metadata::fetch()?;

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

fn cargo_cmd() -> ffi::OsString {
    env::var_os("CARGO").unwrap_or_else(|| ffi::OsString::from("cargo"))
}

#[derive(Eq, PartialEq)]
pub enum TestOutcome {
    Pass,
    Fail(process::ExitStatus),
}
