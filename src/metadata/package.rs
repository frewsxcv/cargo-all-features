use super::Dependency;
use crate::metadata::CargoAllFeatures;
use crate::runner::Runner;
use crate::types::FeatureList;
use crate::CargoCommand;
use crate::Errors;
use crate::Options;
use crate::Outcome;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::path;
use validator::Validate;
use validator::ValidationError;
use std::hash::Hasher;

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
    // Validation used by validator
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

    // Fetches optional dependencies
    fn optional_dependencies(&self) -> impl ParallelIterator<Item = &String> + '_ {
        self.dependencies
            .par_iter()
            .filter_map(Option::<&String>::from)
    }

    // Fetches features from feature_map
    fn features(&self) -> impl ParallelIterator<Item = &String> {
        self.features
            .par_iter()
            .filter(|(key, _)| key != &"default")
            // Some crates use "__" to indicate internal features
            .filter(|(key, _)| !key.starts_with("__"))
            .map(|e| e.0)
    }

    // Fetch features sets
    pub fn feature_sets(&self) -> Vec<FeatureList<&String>> {
        let mut features = vec![];

        // Closure to check if is in deny list
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

        // Clippy screams due to the `.map` and `.unwrap_or` as they seem complex
        #[allow(clippy::blocks_in_if_conditions)]
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

    // Run command on current package for all features
    pub fn run_on_all_features(
        &self,
        command: &CargoCommand,
        arguments: &[String],
        options: Option<&Options>,
    ) -> Result<Outcome, Errors> {
        // Closure to execute command on feature sets
        let run = |chunked_feature_sets: std::slice::Iter<FeatureList<&String>>| {
            for feature_set in chunked_feature_sets {
                let mut runner = Runner::new(
                    command,
                    &self.name,
                    feature_set,
                    self.manifest_path
                        .parent()
                        .expect("could not find parent of cargo manifest path"),
                    arguments,
                    options,
                );

                match runner.run()? {
                    Outcome::Pass => (),
                    // Fail fast if we encounter a test failure
                    t @ Outcome::Fail(_) => return Ok(t),
                }
            }

            // If everything goes well, pass
            Ok(Outcome::Pass)
        };

        // Simple function to calculate hash
        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        // Get feature sets and sort their lists
        let mut feature_sets: Vec<_> = self
            .feature_sets()
            .into_par_iter()
            .map(|mut e| {
                e.par_sort_by(|a, b| calculate_hash(b).cmp(&calculate_hash(a)));
                e
            })
            .collect();

        // Sort feature sets
        feature_sets.par_sort_by(|a, b| calculate_hash(&b.0).cmp(&calculate_hash(&a.0)));

        // Either use chunks
        if let Some(options) = options {
            if let Some(chunk) = options.chunk {
                if let Some(chunks) = options.chunks {
                    if feature_sets.len() >= chunks && chunk <= chunks {
                        // Checks how many for each chunk
                        let len = feature_sets.len();
                        let (quo, rem) = (len / chunks, len % chunks);

                        // Get items for current chunk
                        let mut items = feature_sets
                            .chunks(quo)
                            .collect::<Vec<_>>()
                            .get(chunk - 1)
                            .unwrap()
                            .to_vec();

                        // Check if there are remaining ones, if so adds 1
                        if chunk <= rem {
                            items.push(
                                feature_sets[(chunks * quo)..]
                                    .get(chunk - 1)
                                    .unwrap()
                                    .to_owned(),
                            );
                        }

                        return run(items.iter());
                    } else {
                        // TODO Error
                    }
                }
            }
        }
        // Else just run on all
        return run(feature_sets.iter());
    }
}
