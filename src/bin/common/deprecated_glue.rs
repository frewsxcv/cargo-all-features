use cargo_all_features::{run as run_main, runner::CargoCommand};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))
        .unwrap();
    print!("warning");
    stdout.reset().unwrap();
    println!(
        ": the command `cargo {}` may be deprecated, please use `cargo all-features build`",
        name
    );

    if let Err(error) = run_main(command, &arguments) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .unwrap();
        print!("error");
        stdout.reset().unwrap();

        println!(": {}", error);
    }
}
