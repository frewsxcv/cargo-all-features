use cargo_all_features::{run as run_main, runner::CargoCommand, toolchain::CommandTarget};
use yansi::Paint;

// Glue code to run `cargo build-all-features`, etc. with same logic as `cargo all-features build`
pub fn run(command: CargoCommand) {
    let name: String = env!("CARGO_BIN_NAME").replace("cargo-", "");
    let arguments: Vec<String> = std::env::args()
        .skip(
            if std::env::args().nth(1).unwrap_or_else(|| "".to_string()) == name {
                2
            } else {
                1
            },
        )
        .collect();

    println!(
        "{}: the command `cargo {}` may be deprecated, please use `cargo all-features build`",
        Paint::yellow("warning").bold(),
        name
    );

    if let Err(error) = run_main(command, &arguments, None, CommandTarget::Cargo) {
        println!("{}: {}", Paint::red("error").bold(), error);
    }
}
