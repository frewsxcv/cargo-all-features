use crate::CargoCommand;
use crate::CargoCommandOrigin;
use rayon::prelude::*;
use std::collections::HashMap;

/// Static list of flags not allowed because they can interfere with the commands initiated by this crate
pub const FORBIDDEN_FLAGS: [&str; 20] = [
    "--no-default-features",
    "--features",
    "--bin",
    "--lib",
    "-p",
    "--bins",
    "--workspace",
    "--example",
    "--examples",
    "--test",
    "--tests",
    "--bench",
    "--benches",
    "--all-targets",
    "--manifest-path",
    "--all",
    "--exclude",
    "--bins",
    "--libs",
    "--color",
];

lazy_static::lazy_static! {
    /// Static table of commands and their origin
    /// This is needed to make sure commands are installed
    pub static ref COMMAND_ORIGIN_LOOKUP_MAP: HashMap<CargoCommand, CargoCommandOrigin> = [
        (CargoCommand::Build, CargoCommandOrigin::FirstParty),
        (CargoCommand::Check, CargoCommandOrigin::FirstParty),
        (CargoCommand::Test, CargoCommandOrigin::FirstParty),
        (CargoCommand::Bench, CargoCommandOrigin::FirstParty),
        (CargoCommand::MiriTest, CargoCommandOrigin::RustUpComponent {
            name: "miri",
            help_url: "https://github.com/rust-lang/miri"
        }),
        (CargoCommand::Udeps, CargoCommandOrigin::ThirdPartyCrate {
            name: "cargo-udeps",
            help_url: "https://github.com/est31/cargo-udeps"
        }),
        (CargoCommand::Tarpaulin, CargoCommandOrigin::ThirdPartyCrate {
            name: "cargo-tarpaulin",
            help_url: "https://github.com/xd009642/tarpaulin"
        }),
        (CargoCommand::Nextest, CargoCommandOrigin::ThirdPartyCrate {
            name: "cargo-nextest",
            help_url: "https://github.com/nextest-rs/nextest"
        }),
    ].par_iter().copied().collect();
}
