use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use regex::Regex;
use std::process::Command;

#[test]
fn simple() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["C"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "C"],
        vec!["A", "oDepB"],
        vec!["C", "oDepB"],
        vec!["A", "B", "C"],
        vec!["A", "B", "oDepB"],
        vec!["A", "C", "oDepB"],
        vec!["A", "B", "C", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn skip_sets_1() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_feature_sets = [
            ["C"],
        ]
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "oDepB"],
        vec!["A", "B", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn skip_sets_2() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_feature_sets = [
            ["oDepB", "C"],
        ]
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["C"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "C"],
        vec!["A", "oDepB"],
        vec!["A", "B", "C"],
        vec!["A", "B", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn skip_sets_3() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_feature_sets = [
            ["oDepB", "B", "C"],
        ]
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["C"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "C"],
        vec!["A", "oDepB"],
        vec!["C", "oDepB"],
        vec!["A", "B", "C"],
        vec!["A", "B", "oDepB"],
        vec!["A", "C", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn skip_opt_deps() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_optional_dependencies = true
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["C"],
        vec!["A", "B"],
        vec!["A", "C"],
        vec!["A", "B", "C"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn allowlist() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        allowlist = ["A", "oDepB"]
    "#;
    let valid_feature_sets = vec![vec![], vec!["A"], vec!["oDepB"], vec!["A", "oDepB"]];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn denylist() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        denylist = ["C"]
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "oDepB"],
        vec!["A", "B", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn extra_feats() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_optional_dependencies = true
        extra_features = ["oDepB"]
    "#;
    let valid_feature_sets = vec![
        vec![],
        vec!["A"],
        vec!["C"],
        vec!["oDepB"],
        vec!["A", "B"],
        vec!["A", "C"],
        vec!["A", "oDepB"],
        vec!["C", "oDepB"],
        vec!["A", "B", "C"],
        vec!["A", "B", "oDepB"],
        vec!["A", "C", "oDepB"],
        vec!["A", "B", "C", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn always_include() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        #skip_optional_dependencies = true
        denylist = ["C"]
        always_include_features = ["A"]
    "#;
    let valid_feature_sets = vec![
        vec!["A"],
        vec!["A", "B"],
        vec!["A", "oDepB"],
        vec!["A", "B", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn skip_sets_with_always_include() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_feature_sets = [["A", "B"]]
        always_include_features = ["A"]
    "#;
    let valid_feature_sets = vec![
        vec!["A"],
        vec!["A", "C"],
        vec!["A", "oDepB"],
        vec!["A", "C", "oDepB"],
    ];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn allowlist_with_skip_opt_deps() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
        [features]
        A = []
        B = ["A"]
        C = ["dep:optDepC"]

        [dependencies]
        fixDepA = {path = "fixDepA"}
        oDepB = {path = "optDepB", package = "optDepB", optional = true}
        optDepC = {path = "optDepC", optional = true}

        [package.metadata.cargo-all-features]
        skip_optional_dependencies = true
        allowlist = ["A", "B", "oDepB"]
    "#;
    let valid_feature_sets = vec![vec![], vec!["A"], vec!["A", "B"]];
    test_dummy_crate_setup(
        feats_deps_allfeatssettings_section,
        valid_feature_sets,
        None,
    )
}

#[test]
fn conflicting_rules() -> Result<(), Box<dyn std::error::Error>> {
    let feats_deps_allfeatssettings_section = r#"
[features]
A = []
B = ["A"]
C = ["dep:optDepC"]

[dependencies]
fixDepA = {path = "fixDepA"}
oDepB = {path = "optDepB", package = "optDepB", optional = true}
optDepC = {path = "optDepC", optional = true}

[package.metadata.cargo-all-features]
skip_feature_sets = [["A", "B"]]
always_include_features = ["B"]
"#;
    test_dummy_crate_setup(feats_deps_allfeatssettings_section, vec![], None)
}

fn cargo_dep_setup(
    dep_name: &str,
    cwd: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("new").arg(dep_name).current_dir(cwd);
    cmd.output()?;
    Ok(())
}

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

fn get_tested_feature_sets_from_output(stdout: &str) -> Vec<Vec<&str>> {
    let re = Regex::new(r"(?m)^.*Testing.*crate=testdummy features=\[(.*)\]$").unwrap();

    let mut ans = vec![];
    for (_, [comma_sep_features]) in re.captures_iter(stdout).map(|c| c.extract()) {
        ans.push(comma_sep_features.split(',').collect());
    }
    ans
}

fn test_dummy_crate_setup(
    feats_deps_allfeatssettings_section: &str,
    valid_feature_sets: Vec<Vec<&str>>,
    expected_error: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    println!("{:?}", temp.path());
    cargo_dep_setup("fixDepA", temp.path())?;
    cargo_dep_setup("optDepB", temp.path())?;
    cargo_dep_setup("optDepC", temp.path())?;
    let cargotoml = temp.child("Cargo.toml");
    cargotoml.write_str(&format!(
        r#"
        [package]
        name = "testdummy"
        version = "0.1.0"
        {feats_deps_allfeatssettings_section}
        "#
    ))?;
    temp.child("src/main.rs").touch()?;

    let mut cmd = Command::cargo_bin("cargo-test-all-features")?;
    cmd.arg("test-all-features");
    cmd.current_dir(temp.path());

    cmd.env("CARGO_INCREMENTAL", "0");
    cmd.env("RUSTFLAGS", "-Cinstrument-coverage");
    cmd.env(
        "LLVM_PROFILE_FILE",
        "/home/david/code/github/cargo-all-features/target/profraw/cargo-test-%p-%m.profraw",
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
