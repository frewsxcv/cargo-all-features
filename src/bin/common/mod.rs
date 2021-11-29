use std::ffi::OsString;
use std::{
    env,
    process::{self, Command},
};

pub fn run_deprecated(arg: &str) -> ! {
    let mut cmd = Command::new(cargo_path());
    cmd.arg("all-features").arg(arg);

    let mut cmd_args = env::args_os().skip(1);
    if let Some(first) = cmd_args.next() {
        if let Some(first) = first.to_str() {
            if !first.ends_with("-all-features") {
                cmd.arg(first);
            }
        }
    }

    let exit_status = cmd
        .args(cmd_args)
        .spawn()
        .expect("failed to spawn child process")
        .wait()
        .expect("failed to wait for child process");

    eprintln!(
        "********************DEPRECATION NOTICE********************\n\
        The cargo-all-features crate is switching to using a single\n\
        binary distribution. The currently executing binary will be\n\
        removed in a future release. Instead use:\n\
        \tcargo all-features {}\n\
        ***********************************************************",
        arg
    );

    process::exit(exit_status.code().unwrap_or(-1));
}

fn cargo_path() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}
