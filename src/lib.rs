use clap::{error::ErrorKind, Command, Parser, ValueEnum};
use std::{env, error, ffi, process};

pub mod cargo_metadata;
pub mod features_finder;
pub mod test_runner;
mod types;

#[derive(Parser, Clone)]
#[command(author, version, about = "See https://crates.io/crates/cargo-all-features", long_about = None)]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
/// The cargo wrapper so that `cargo check-all-features ...` will work, since it internally invokes `check-all-features` with itself
/// as the first argument
enum CargoCli {
    #[command(name = "all-features")]
    // Backward compatibility
    #[command(alias = "check-all-features")]
    #[command(alias = "test-all-features")]
    #[command(alias = "build-all-features")]
    Subcommand(Cli),
}

#[derive(Parser, Clone)]
#[command(author, version, about = "See https://crates.io/crates/cargo-all-features", long_about = None)]
struct Cli {
    #[arg(
        long,
        default_value_t = 1,
        requires = "chunk",
        help = "Split the workspace into n chunks, each chunk containing a roughly equal number of crates"
    )]
    n_chunks: usize,
    #[arg(
        long,
        default_value_t = 1,
        requires = "n_chunks",
        help = "Which chunk to test, indexed at 1"
    )]
    chunk: usize,

    // Backward compatibility: keep the field optional
    cargo_command: Option<String>,

    #[arg(
        long,
        value_enum,
        default_value_t = ChunkGranularity::Package,
        help = "Chunk granularity: `package` to chunk by crate, `feature` to chunk by (crate, feature-set) tuples"
    )]
    chunk_granularity: ChunkGranularity,

    #[arg(
        help = "arguments to pass down to cargo",
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    cargo_args: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ChunkGranularity {
    Package,
    Feature,
}

#[derive(Clone, Debug)]
enum WorkItem {
    PackageOnly(cargo_metadata::Package),
    PackageFeature(cargo_metadata::Package, types::FeatureList),
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

pub fn run() -> Result<(), Box<dyn error::Error>> {
    let CargoCli::Subcommand(mut cli) = CargoCli::parse();

    let mut cmd = Command::new("cargo-all-features");

    // Backward compatibility.
    // Safety: Cargo always passes the command as second argument.
    let cargo_command = std::env::args().nth(1).unwrap();

    // Backward compatibility.
    // Check if older commands is used, use cli.cargo_command as an argument, and extract the
    // command.
    // Otherwise, a command should be provided for `cargo all-features <command>`
    let cargo_command = if let Some(cargo_command) = cargo_command.strip_suffix("-all-features") {
        if let Some(arg) = cli.cargo_command {
            cli.cargo_args.insert(0, arg);
        }
        cargo_command.into()
    } else {
        if cli.cargo_command.is_none() {
            cmd.error(
                ErrorKind::InvalidValue,
                "A cargo command is needed, e.g. check, test, build, clippy and ...",
            )
            .print()?;
            process::exit(1);
        }
        cli.cargo_command.unwrap()
    };

    if cli.chunk > cli.n_chunks || cli.chunk < 1 {
        cmd.error(
            ErrorKind::InvalidValue,
            "Must not ask for chunks out of bounds",
        )
        .print()?;
        process::exit(1);
    }

    if cli.n_chunks == 0 {
        cmd.error(ErrorKind::InvalidValue, "--n-chunks must be at least 1")
            .print()?;
        process::exit(1)
    }

    let packages = determine_packages_to_test()?;

    // Build the list of work items. If split_by_feature is set, expand each package into
    // (package, feature-set) tuples. Otherwise operate on packages as a whole.
    let work_items: Vec<WorkItem> = match &cli.chunk_granularity {
        ChunkGranularity::Feature => packages
            .into_iter()
            .flat_map(|package| {
                features_finder::fetch_feature_sets(&package)
                    .into_iter()
                    .map(|f| WorkItem::PackageFeature(package.clone(), f))
                    .collect::<Vec<_>>()
            })
            .collect(),
        ChunkGranularity::Package => packages.into_iter().map(WorkItem::PackageOnly).collect(),
    };

    // chunks() takes a chunk size, not a number of chunks
    // we must adjust to deal with the fact that if things are not a perfect multiple,
    // len / n_chunks will end up with an uncounted remainder chunk
    let mut chunk_size = work_items.len() / cli.n_chunks;
    #[allow(clippy::manual_is_multiple_of)]
    if work_items.len() % cli.n_chunks != 0 {
        chunk_size += 1;
    }

    // - 1 since we are 1-indexing
    let chunk = if let Some(chunk) = work_items.chunks(chunk_size).nth(cli.chunk - 1) {
        chunk
    } else {
        println!("Chunk is empty (did you ask for more chunks than there are packages?");
        return Ok(());
    };
    if cli.n_chunks != 1 {
        print_chunk_info(
            cli.chunk_granularity,
            cli.chunk,
            cli.n_chunks,
            chunk_size,
            chunk,
        );
    }

    for item in chunk {
        let outcome = match item {
            WorkItem::PackageOnly(package) => {
                test_all_features_for_package(package, cargo_command.clone(), &cli.cargo_args)
            }
            WorkItem::PackageFeature(package, feature_set) => test_one_feature_for_package(
                package,
                feature_set,
                cargo_command.clone(),
                &cli.cargo_args,
            ),
        }?;
        if let TestOutcome::Fail(exit_status) = outcome {
            process::exit(exit_status.code().unwrap());
        }
    }

    Ok(())
}

fn print_chunk_info(
    chunk_granularity: ChunkGranularity,
    chunk_index: usize,
    n_chunks: usize,
    chunk_size: usize,
    chunk: &[WorkItem],
) {
    let (chunk_size, packages) = match chunk_granularity {
        ChunkGranularity::Feature => {
            let packages: String = chunk
                .iter()
                .map(|w| match w {
                    WorkItem::PackageFeature(package, feature_set) => {
                        let feature_list = if feature_set.is_empty() {
                            "<none>".to_string()
                        } else {
                            feature_set
                                .iter()
                                .map(|f| f.as_ref().to_string())
                                .collect::<Vec<_>>()
                                .join("+")
                        };
                        format!("{} [{}]", package.name, feature_list)
                    }
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            (chunk.len(), packages)
        }
        ChunkGranularity::Package => {
            let packages: String = chunk
                .iter()
                .map(|w| match w {
                    WorkItem::PackageOnly(p) | WorkItem::PackageFeature(p, _) => p.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(",");
            (chunk_size, packages)
        }
    };

    println!(
        "Running on chunk {} out of {} ({chunk_size} packages: {packages})",
        chunk_index, n_chunks
    );
}

fn test_all_features_for_package(
    package: &cargo_metadata::Package,
    command: String,
    cargo_args: &[String],
) -> Result<TestOutcome, Box<dyn error::Error>> {
    let feature_sets = crate::features_finder::fetch_feature_sets(package);

    for feature_set in feature_sets {
        let outcome =
            test_one_feature_for_package(package, &feature_set, command.clone(), cargo_args)?;

        match outcome {
            TestOutcome::Pass => (),
            // Fail fast if we encounter a test failure
            t @ TestOutcome::Fail(_) => return Ok(t),
        }
    }

    Ok(TestOutcome::Pass)
}

fn test_one_feature_for_package(
    package: &cargo_metadata::Package,
    feature_set: &types::FeatureList,
    command: String,
    cargo_args: &[String],
) -> Result<TestOutcome, Box<dyn error::Error>> {
    let mut test_runner = crate::test_runner::TestRunner::new(
        command.clone(),
        package.name.clone(),
        feature_set.clone(),
        cargo_args,
        package
            .manifest_path
            .parent()
            .expect("could not find parent of cargo manifest path")
            .to_owned(),
    );

    test_runner.run()
}

fn determine_packages_to_test() -> Result<Vec<cargo_metadata::Package>, Box<dyn error::Error>> {
    let current_dir = env::current_dir()?;
    let metadata = cargo_metadata::fetch()?;

    Ok(if current_dir == metadata.workspace_root {
        metadata
            .packages
            .iter()
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .filter(|package| !package.skip_package)
            .cloned()
            .collect::<Vec<cargo_metadata::Package>>()
    } else {
        vec![metadata
            .packages
            .iter()
            .find(|package| package.manifest_path.parent() == Some(&current_dir))
            .expect("Could not find cargo package in metadata")
            .to_owned()]
    })
}

fn cargo_cmd() -> ffi::OsString {
    env::var_os("CARGO").unwrap_or_else(|| ffi::OsString::from("cargo"))
}

#[derive(Eq, PartialEq)]
pub enum TestOutcome {
    Pass,
    Fail(process::ExitStatus),
}
