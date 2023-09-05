use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use regex::Regex;
use std::process::Command;

#[test]
fn simple() -> Result<(), Box<dyn std::error::Error>> {
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["B"],
        vec!["C"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "C"],
        vec!["A", "oDepB"],
        vec!["B", "C"],
        vec!["B", "oDepB"],
        vec!["C", "oDepB"],
        vec!["A", "B", "C"],
        vec!["A", "B", "oDepB"],
        vec!["A", "C", "oDepB"],
        vec!["B", "C", "oDepB"],
        vec!["A", "B", "C", "oDepB"],
    ];
    test_settings("", valid_feature_sets, None)
}

/*
The NormStr trait provides a normalize method which produces a string representation from itself.
The normalization in these test cases refers to the order of features in lists. The order shall be ignored, therefor
the normalization produces the same string output for differently ordered sets containing the same elements.
The DELIMS, PREFIX, and POSTFIX just provide formating guides for a different list-depth.
E.g. [[A,B], [C,D]] would produce the string (A B), (C D).
*/
trait NormStr
where
    Self: Sized,
{
    const DELIMS: [&'static str; 3] = ["), (", " ", "/"];
    const PREFIX: [&'static str; 3] = ["(", "", ""];
    const POSTFIX: [&'static str; 3] = [")", "", ""];
    fn normalize(self) -> String {
        self.depthwise_normalize(0)
    }

    fn depthwise_normalize(self, depth: usize) -> String;
}

impl<T> NormStr for Vec<T>
where
    T: NormStr + std::fmt::Debug,
{
    fn depthwise_normalize(self, depth: usize) -> String {
        let mut v: Vec<_> = self
            .into_iter()
            .map(|e| e.depthwise_normalize(depth + 1))
            .collect();
        v.sort();
        Self::PREFIX[depth].to_owned() + &v.join(Self::DELIMS[depth]) + Self::POSTFIX[depth]
    }
}

impl NormStr for &str {
    fn depthwise_normalize(self, _depth: usize) -> String {
        self.to_owned()
    }
}

/*
Given a path for the crate (cwd) and a dependency name,
this function creates a local dummy crate which can be used as dependency,
using the path attribute in Cargo.toml.
E.g.
[dependencies]
dep_name = {path="dep_name"}
*/
fn cargo_dep_setup(
    dep_name: &str,
    cwd: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("new").arg(dep_name).current_dir(cwd);
    cmd.output()?;
    Ok(())
}

/*
Using cargo-all-features produces an output of each test or build with a headline,
denoting the crate and activated features, e.g. Testing crate=testdummy features=[A,B].
This function expects such a output and filters all the feature sets which were discovered in the output.
*/
fn get_tested_feature_sets_from_output(stdout: &str) -> Vec<Vec<&str>> {
    let re = Regex::new(r"(?m)^.*Testing.*crate=testdummy features=\[(.*)\]$").unwrap();

    let mut ans = vec![];
    for (_, [comma_sep_features]) in re.captures_iter(stdout).map(|c| c.extract()) {
        ans.push(comma_sep_features.split(',').collect());
    }
    ans
}

/*
Setup a compilable dummy crate with no implementation and return the path to this crate layout.
*/
fn dummy_crate_setup(settings: &str) -> Result<assert_fs::TempDir, Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    cargo_dep_setup("fixDepA", temp.path())?;
    cargo_dep_setup("optDepB", temp.path())?;
    cargo_dep_setup("optDepC", temp.path())?;
    let cargotoml = temp.child("Cargo.toml");
    cargotoml.write_str(&format!(
        r#"
        [package]
        name = "testdummy"
        version = "0.1.0"

        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {{path = "fixDepA"}}
        oDepB = {{path = "optDepB", package = "optDepB", optional = true}}
        optDepC = {{path = "optDepC", optional = true}}

        [package.metadata.cargo-all-features]
        {settings}
        "#
    ))?;
    temp.child("src/main.rs").touch()?;
    Ok(temp)
}

/*
Test the given settings for cargo-all-features.
If an error message is provided, expect cargo test-all-features to fail with this message.
Otherwise expect the normalized set of feature sets to be the same as the given ground truth input.
*/
fn test_settings(
    settings: &str,
    valid_feature_sets: Vec<Vec<&str>>,
    expected_error: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = dummy_crate_setup(settings)?;
    let mut cmd = Command::cargo_bin("cargo-test-all-features")?;
    cmd.arg("test-all-features");
    cmd.current_dir(temp.path());

    // add flags for producing also a coverage report, see ci/test_and_coverage.bash
    cmd.env("CARGO_INCREMENTAL", "0");
    cmd.env("RUSTFLAGS", "-Cinstrument-coverage");
    cmd.env(
        "LLVM_PROFILE_FILE",
        format!(
            "{}/target/profraw/cargo-test-%p-%m.profraw",
            std::env::current_dir()?.display()
        ),
    );

    if let Some(err_msg) = expected_error {
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains(err_msg));
    } else {
        let stdout = String::from_utf8(cmd.assert().success().get_output().stdout.clone()).unwrap();
        let produced_feat_sets = get_tested_feature_sets_from_output(&stdout);
        assert_eq!(
            valid_feature_sets.normalize(),
            produced_feat_sets.normalize()
        );
    }
    temp.close()?;
    Ok(())
}
