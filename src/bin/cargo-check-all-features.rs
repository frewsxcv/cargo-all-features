use cargo_all_features::{run, test_runner::CargoCommand};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    run(CargoCommand::Check)
}
