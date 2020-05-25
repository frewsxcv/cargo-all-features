use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    cargo_all_features::run(cargo_all_features::test_runner::CargoCommand::Test)
}
