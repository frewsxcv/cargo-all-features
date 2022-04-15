use cargo_all_features::runner::CargoCommand;
mod common;

fn main() {
    common::deprecated_glue::run(CargoCommand::Build);
}
