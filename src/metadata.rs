use crate::runner::{CargoCommand, Runner};
use rayon::iter::ParallelIterator;
use serde::Deserialize;

use crate::types::FeatureList;
use crate::{Errors, TestOutcome};
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::{
    path,
    process::{Command, Stdio},
};
use validator::{Validate, ValidationError};

pub struct MetaTree {
    meta_data: MetaData,
}

impl MetaTree {
    pub fn new() -> Result<Self, Errors> {
        let mut command = Command::new(crate::cargo_cmd());

        command.args(&["metadata", "--format-version", "1"]);

        let output = command.stderr(Stdio::inherit()).output()?;

        if !output.status.success() {
            return Err(Errors::CargoMetaDataNonZeroStatus {
                status: output.status,
            });
        }

        let meta_data: MetaData = serde_json::from_slice(&output.stdout)?;

        for package in &meta_data.packages {
            package.validate()?
        }

        Ok(Self { meta_data })
    }

    pub fn meta_data(&self) -> &MetaData {
        &self.meta_data
    }
}

#[derive(Clone, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub rename: Option<String>,
    pub optional: bool,
}

impl<'a> From<&'a Dependency> for Option<&'a String> {
    fn from(dependency: &'a Dependency) -> Self {
        dependency
            .optional
            .then(|| dependency.rename.as_ref().unwrap_or(&dependency.name))
    }
}

#[derive(Clone, Deserialize, Validate)]
#[validate(schema(function = "Self::validate"))]
pub struct Package {
    pub id: String,
    pub name: String,
    pub manifest_path: path::PathBuf,
    pub dependencies: Vec<Dependency>,
    pub features: HashMap<String, FeatureList>,
    #[serde(rename = "cargo-all-features")]
    pub cargo_all_features: Option<CargoAllFeatures>,
}

impl Package {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(config) = &self.cargo_all_features {
            if let Some(allow_ist) = &config.allowlist {
                if !allow_ist.is_empty() {
                    if let Some(deny_list) = &config.denylist {
                        if !deny_list.is_empty() {
                            let mut error = ValidationError::new(
                                "Package has both `allowlist` and `denylist` keys",
                            );
                            error.add_param("name".into(), &self.name);

                            return Err(error);
                        }
                    }
                    if let Some(extra_features) = &config.extra_features {
                        if !extra_features.is_empty() {
                            let mut error = ValidationError::new(
                                "Package has both `allowlist` and `extra_features` keys",
                            );
                            error.add_param("name".into(), &self.name);

                            return Err(error);
                        }
                    }
                    if config.skip_optional_dependencies.is_some() {
                        let mut error = ValidationError::new(
                            "Package has both `allowlist` and `skip_optional_dependencies` keys",
                        );
                        error.add_param("name".into(), &self.name);

                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    fn optional_dependencies(&self) -> impl ParallelIterator<Item = &String> + '_ {
        self.dependencies
            .par_iter()
            .filter_map(Option::<&String>::from)
    }

    fn features(&self) -> impl ParallelIterator<Item = &String> {
        self.features
            .par_iter()
            .filter(|(key, _)| key != &"default")
            // Some crates use "__" to indicate internal features
            .filter(|(key, _)| !key.starts_with("__"))
            .map(|e| e.0)
    }

    pub fn feature_sets(&self) -> Vec<FeatureList<&String>> {
        let mut features = vec![];

        let filter_denylist = |f: &String| {
            self.cargo_all_features
                .as_ref()
                .map(|mf| !mf.denylist.as_ref().map(|e| e.contains(f)).unwrap_or(false))
                .unwrap_or(true)
        };

        // Pre-Cargo 1.60, `cargo metadata` would only report explicitly specified features
        // Post-Cargo 1.60, with the advent of dep: syntax (https://doconfig.rust-lang.org/nightly/cargo/reference/features.html#optional-dependencies),
        // `cargo metadata` will unconditionally report all features, including ones autoderived from optional dependencies.
        //
        // We try to handle both cases by searching for any use of the `dep:` keyword, and treating features that have been marked as optional AND
        // exist simply as `foo = ["dep:foo"]` as implicit optional dependencies; even if they were explicitly specified. Upstream bug at
        // https://github.com/rust-lang/cargo/issues/10543
        let optional_deps: Vec<_> = self
            .optional_dependencies()
            .filter(|e| filter_denylist(e))
            .collect();
        let mut optional_denylist = HashSet::new();
        let mut found_dep = false;
        for dep in &optional_deps {
            if let Some(sub) = self.features.get(&**dep) {
                if sub.par_iter().any(|v| v.starts_with("dep:")) {
                    found_dep = true;
                    if sub.len() == 1 && sub[0] == format!("dep:{}", dep) {
                        optional_denylist.insert(dep);
                    }
                }
            }
        }

        if self
            .cargo_all_features
            .as_ref()
            .map(|config| {
                config
                    .allowlist
                    .as_ref()
                    .map(|e| e.is_empty())
                    .unwrap_or(true)
            })
            .unwrap_or(true)
        {
            // This handles the pre-1.60 case
            if !self
                .cargo_all_features
                .as_ref()
                .map(|config| config.skip_optional_dependencies.unwrap_or(false))
                .unwrap_or(false)
                && !found_dep
            {
                features.par_extend(optional_deps.par_iter().filter(|f| filter_denylist(f)));
            }
            features.par_extend(
                self.features()
                    .filter(|f| filter_denylist(f))
                    // This handles the post-1.60 case
                    .filter(|f| {
                        !self
                            .cargo_all_features
                            .as_ref()
                            .map(|config| config.skip_optional_dependencies.unwrap_or(false))
                            .unwrap_or(false)
                            || !optional_denylist.contains(f)
                    }),
            );

            if let Some(config) = self.cargo_all_features.as_ref() {
                if let Some(extra_features) = &config.extra_features {
                    features.par_extend(extra_features.par_iter().filter(|e| filter_denylist(e)));
                }
            }
        } else {
            // allowlist cannot be mixed with denylist or any of the other above options,
            // no need to filter
            if let Some(config) = self.cargo_all_features.as_ref() {
                if let Some(allow_list) = &config.allowlist {
                    features.par_extend(allow_list.par_iter())
                }
            }
        };

        let mut feature_sets = vec![];

        for n in 0..=features.len() {
            'outer: for feature_set in features.iter().combinations(n) {
                if let Some(config) = self.cargo_all_features.as_ref() {
                    if let Some(skip_feature_sets) = &config.skip_feature_sets {
                        'inner: for skip_feature_set in skip_feature_sets {
                            for feature in skip_feature_set.iter() {
                                if !feature_set.contains(&&feature) {
                                    // skip_feature_set does not match
                                    continue 'inner;
                                }
                            }
                            // skip_feature_set matches: do not add it to feature_sets
                            continue 'outer;
                        }
                    }
                }
                // `copied` needed as else the data would be owned by this function
                feature_sets.push(FeatureList::from_iter(feature_set.into_iter().copied()));
            }
        }

        feature_sets
    }

    pub fn run_on_all_features(
        &self,
        command: &CargoCommand,
        arguments: &[String],
    ) -> Result<TestOutcome, Errors> {
        for feature_set in self.feature_sets() {
            let outcome = Runner::new(
                command,
                &self.name,
                feature_set,
                self.manifest_path
                    .parent()
                    .expect("could not find parent of cargo manifest path"),
                arguments,
            )
            .run()?;

            match outcome {
                TestOutcome::Pass => (),
                // Fail fast if we encounter a test failure
                t @ TestOutcome::Fail(_) => return Ok(t),
            }
        }

        Ok(TestOutcome::Pass)
    }
}

#[derive(Clone, Deserialize, Validate)]
pub struct CargoAllFeatures {
    pub skip_feature_sets: Option<Vec<FeatureList>>,
    pub skip_optional_dependencies: Option<bool>,
    pub extra_features: Option<FeatureList>,
    pub allowlist: Option<FeatureList>,
    pub denylist: Option<HashSet<String>>,
}

#[derive(Clone, Deserialize)]
pub struct MetaData {
    pub workspace_root: PathBuf,
    pub workspace_members: Vec<String>,
    pub packages: Vec<Package>,
}

impl<'a> MetaData {
    pub fn determine_packages_to_run_on(&'a self) -> Result<Vec<&'a Package>, Errors> {
        let current_dir = env::current_dir()?;

        Ok(if current_dir == self.workspace_root {
            self.packages
                .par_iter()
                .filter(|package| self.workspace_members.contains(&package.id))
                .collect::<Vec<_>>()
        } else {
            vec![self
                .packages
                .par_iter()
                .find_any(|package| package.manifest_path.parent() == Some(&current_dir))
                .expect("Could not find cargo package in metadata")]
        })
    }
}
