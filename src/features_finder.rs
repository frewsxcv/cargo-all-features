use crate::{
    cargo_metadata::Dependency,
    types::{Feature, FeatureList},
};
use itertools::Itertools;
use std::collections::HashSet;

pub fn fetch_feature_sets(package: &crate::cargo_metadata::Package) -> Vec<FeatureList> {
    let mut features = FeatureList::default();

    let filter_denylist = |f: &Feature| !package.denylist.contains(f);

    let mut implicit_features = HashSet::<&str>::new();
    let mut optional_dep_used_with_dep_syntax_outside_of_implicit_feature = HashSet::new();

    for (feature, implied_features) in &package.feature_map {
        for implied_dep in implied_features
            .iter()
            .filter_map(|v| v.strip_prefix("dep:"))
        {
            if implied_features.len() == 1 && implied_dep == feature {
                // Feature of the shape foo = ["dep:foo"]
                implicit_features.insert(feature);
            } else {
                optional_dep_used_with_dep_syntax_outside_of_implicit_feature.insert(implied_dep);
            }
        }
    }

    // If the dep is used with `dep:` syntax in another feature,
    // it's an explicit feature, because cargo wouldn't generate
    // the implicit feature.
    for x in &optional_dep_used_with_dep_syntax_outside_of_implicit_feature {
        implicit_features.remove(x);
    }

    if package.allowlist.is_empty() {
        if !package.skip_optional_dependencies {
            features.extend(
                fetch_optional_dependencies(package)
                    .filter(filter_denylist)
                    .filter(|f: &Feature| {
                        !optional_dep_used_with_dep_syntax_outside_of_implicit_feature
                            .contains(f.0.as_str())
                    }),
            );
        }

        features.extend(
            fetch_features(package)
                .filter(filter_denylist)
                .filter(|f: &Feature| !implicit_features.contains(f.0.as_str())),
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

    let max_combination_size = package.max_combination_size.unwrap_or(features.len());
    for n in 0..=max_combination_size {
        'outer: for feature_set in features.iter().combinations(n) {
            for skip_feature_set in &package.skip_feature_sets {
                // skip permutation if skip_feature_set is a subset
                if skip_feature_set.iter().all(|f| feature_set.contains(&f)) {
                    continue 'outer;
                }
            }
            // skip permutations which do not contain at least one feature per set of always_include_features
            for include_feature_set in &package.always_include_features {
                if include_feature_set
                    .iter()
                    .all(|f| !feature_set.contains(&f))
                {
                    continue 'outer;
                }
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
