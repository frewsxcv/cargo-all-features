use std::{error, process};

fn main() -> Result<(), Box<dyn error::Error>> {
    let packages = cargo_all_features::determine_packages_to_test()?;

    for package in packages {
        let outcome = cargo_all_features::test_all_features_for_package(
            &package,
            cargo_all_features::test_runner::CargoCommand::Test,
        )?;

        if let cargo_all_features::TestOutcome::Fail(exit_status) = outcome {
            process::exit(exit_status.code().unwrap());
        }
    }

    Ok(())
}
