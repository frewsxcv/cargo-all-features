use cargo_all_features::{run, test_runner::CargoCommand};
use clap::{crate_authors, crate_description, crate_license, crate_version, Parser};
use std::error::Error;

#[derive(Parser)]
#[clap(
    name = "cargo all-features",
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
    license = crate_license!(),
)]
struct Opts {
    /// choose which cargo command to run on the feature matrix: build, check, test
    command: CargoCommand,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    run(opts.command)?;
    Ok(())
}
