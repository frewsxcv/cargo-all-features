use cargo_all_features::runner::CargoCommand;
use clap::{Parser, Subcommand};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long)]
    pub chunks: Option<u8>,

    #[clap(long)]
    pub chunk: Option<u8>,

    #[clap(arg_enum)]
    pub command: CargoCommand,

    #[clap(subcommand)]
    pub flags_and_options: Option<FlagsAndOptions>,
}

#[derive(Debug, Subcommand)]
enum FlagsAndOptions {
    #[clap(external_subcommand)]
    External(Vec<String>),
}

fn run_command(command: CargoCommand, args: &[String]) {
    if let Err(error) = cargo_all_features::run(command, args) {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .unwrap();
        print!("error");
        stdout.reset().unwrap();

        println!(": {}", error);
    }
}

pub fn main() {
    let name: String = env!("CARGO_BIN_NAME").replace("cargo-", "");
    let arguments = std::env::args().skip(
        if std::env::args().nth(1).unwrap_or_else(|| "".to_string()) == name {
            1
        } else {
            0
        },
    );

    let args = Cli::parse_from(arguments);

    if let Some(external_command) = args.flags_and_options {
        match external_command {
            FlagsAndOptions::External(commands) => {
                run_command(args.command, &commands);
            }
        }
    } else {
        run_command(args.command, &[]);
    }
}
