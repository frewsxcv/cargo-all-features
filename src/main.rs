use cargo_all_features::{run, test_runner::CargoCommand};
use clap::{crate_authors, crate_description, crate_license, crate_name, crate_version, Parser};
use std::error::Error;

#[derive(Parser)]
#[clap(
    name = crate_name!(),
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
    license = crate_license!(),
    bin_name = "cargo all-features",
    visible_alias = "all-features",
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
