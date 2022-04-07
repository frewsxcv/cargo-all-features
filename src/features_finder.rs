use crate::{
    cargo_metadata::Dependency,
    types::{Feature, FeatureList},
};
use itertools::Itertools;
use std::collections::HashSet;

pub fn fetch_feature_sets(package: &crate::cargo_metadata::Package) -> Vec<FeatureList> {
    let mut features = FeatureList::default();

    let filter_denylist = |f: &Feature| !package.denylist.contains(f);

    // Pre-Cargo 1.60, `cargo metadata` would only report explicitly specified features
    // Post-Cargo 1.60, with the advent of dep: syntax (https://doc.rust-lang.org/nightly/cargo/reference/features.html#optional-dependencies),
    // `cargo metadata` will unconditionally report all features, including ones autoderived from optional dependencies.
    //
    // We try to handle both cases by searching for any use of the `dep:` keyword, and treating features that have been marked as optional AND
    // exist simply as `foo = ["dep:foo"]` as implicit optional dependencies; even if they were explicitly specified. Upstream bug at
    // https://github.com/rust-lang/cargo/issues/10543
    let optional_deps: Vec<_> = fetch_optional_dependencies(package)
        .filter(filter_denylist)
        .collect();
    let mut optional_denylist = HashSet::new();
    let mut found_dep = false;
    for dep in &optional_deps {
        if let Some(sub) = package.feature_map.get(&**dep) {
            if sub.iter().any(|v| v.starts_with("dep:")) {
                found_dep = true;
                if sub.len() == 1 && sub[0] == format!("dep:{}", dep.0) {
                    optional_denylist.insert(dep);
                }
            }
        }
    }

    if package.allowlist.is_empty() {
        // This handles the pre-1.60 case
        if !package.skip_optional_dependencies && !found_dep {
            features.extend(optional_deps.iter().filter(|f| filter_denylist(f)).cloned());
        }
        features.extend(
            fetch_features(package)
                .filter(filter_denylist)
                // This handles the post-1.60 case
                .filter(|f: &Feature| {
                    !package.skip_optional_dependencies || !optional_denylist.contains(f)
                }),
        );

        features.extend(
            package
                .extra_features
                .iter()
                .cloned()
                .filter(filter_denylist),
        );
    } else {
        // allowlist cannot be mixed with denylist or any of the other above options,
        // no need to filter
        features.extend(package.allowlist.iter().cloned())
    };

    let mut feature_sets = vec![];

    for n in 0..=features.len() {
        'outer: for feature_set in features.iter().combinations(n) {
            'inner: for skip_feature_set in &package.skip_feature_sets {
                for feature in skip_feature_set.iter() {
                    if !feature_set.contains(&feature) {
                        // skip_feature_set does not match
                        continue 'inner;
                    }
                }
                // skip_feature_set matches: do not add it to feature_sets
                continue 'outer;
            }
            feature_sets.push(feature_set.into_iter().cloned().collect());
        }
    }

    feature_sets
}

fn fetch_optional_dependencies(
    package: &crate::cargo_metadata::Package,
) -> impl Iterator<Item = Feature> + '_ {
    package
        .dependencies
        .iter()
        .filter_map(Dependency::as_feature)
}

fn fetch_features(package: &crate::cargo_metadata::Package) -> impl Iterator<Item = Feature> + '_ {
    package
        .features
        .iter()
        .filter(|key| key != "default")
        // Some crates use "__" to indicate internal features
        .filter(|key| !key.starts_with("__"))
        .cloned()
}

impl Dependency {
    fn as_feature(&self) -> Option<Feature> {
        self.optional
            .then(|| self.rename.as_ref().unwrap_or(&self.name))
            .cloned()
            .map(Feature)
    }
}
