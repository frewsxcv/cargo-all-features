use cargo_all_features::{run_deprecated, test_runner::CargoCommand};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    run_deprecated(CargoCommand::Test)
}
