use cargo_all_features::{runner::CargoCommand, Options};
use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};
use yansi::Paint;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_BIN_NAME"),
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
    bin_name = "cargo all-features",
    visible_alias = "all-features",
)]
struct Cli {
    #[clap(long, help="The total number of chunks to split into. Only used for calculations", possible_values(["1.."]))]
    pub chunks: Option<usize>,

    #[clap(long, help="The chunk to process", possible_values(["1..<CHUNKS>"]))]
    pub chunk: Option<usize>,

    #[clap(long, help = "If enabled will not execute commands")]
    pub dry_run: bool,

    #[clap(long, help = "If enabled will not disable any coloring")]
    pub no_color: bool,

    #[clap(
        long,
        short,
        help = "If enabled will show command which will or would be executed"
    )]
    pub verbose: bool,

    #[clap(
        long,
        short,
        help = "If enabled will use cross instead of cargo"
    )]
    pub cross: bool,


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

// Runs the command and prints out in rust known error format
fn run_command(command: CargoCommand, args: &[String], options: Option<Options>) {
    if let Err(error) = cargo_all_features::run(command, args, options) {
        println!("{}: {}", Paint::red("error").bold(), error);
    }
}

// Main entrypoint for `cargo all-features`, cli as the frontend
pub fn main() {
    // Name of the cargo subcommand
    let name: String = env!("CARGO_BIN_NAME").replace("cargo-", "");

    // Checking if command is used via cargo or as binary (such as using cargo build --bin all-features)
    let arguments = std::env::args().skip(
        if std::env::args().nth(1).unwrap_or_else(|| "".to_string()) == name {
            1
        } else {
            0
        },
    );

    // Parsing input args
    let args = Cli::parse_from(arguments);

    // Checking if options are specified and transforming them into the libraries business logic
    let mut options = Options {
        no_color: args.no_color,
        dry_run: args.dry_run,
        verbose: args.verbose,
        cross: args.cross,
        chunks: None,
        chunk: None,
    };

    // TODO check if cross is installed, but in library

    // Only if chunk and chunks are set
    if args.chunks.is_some() && args.chunk.is_some() {
        options.chunks = args.chunks;
        options.chunk = args.chunk;
    }

    // Disable color
    if args.no_color {
        Paint::disable();
    }

    // Either run with additional flags and subcommands or without
    if let Some(external_command) = args.flags_and_options {
        match external_command {
            FlagsAndOptions::External(commands) => {
                run_command(args.command, &commands, Some(options));
            }
        }
    } else {
        run_command(args.command, &[], Some(options));
    }
}
