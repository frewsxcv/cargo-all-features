use itertools::Itertools;

pub fn fetch_feature_sets(package: &crate::cargo_metadata::Package) -> Vec<Vec<String>> {
    let mut features = vec![];
    features.append(&mut fetch_optional_dependencies(&package));
    features.append(&mut fetch_features(&package));

    let mut feature_sets = vec![];

    for n in 0..=features.len() {
        for feature_set in features.iter().combinations(n) {
            feature_sets.push(feature_set.iter().map(|n| n.to_string()).collect());
        }
    }

    feature_sets
}

fn fetch_optional_dependencies(package: &crate::cargo_metadata::Package) -> Vec<String> {
    package
        .dependencies
        .iter()
        .filter(|dependency| dependency.optional)
        .map(|dependency| dependency.name.to_string())
        .collect()
}

fn fetch_features(package: &crate::cargo_metadata::Package) -> Vec<String> {
    package
        .features
        .iter()
        .filter(|key| key != &"default")
        .cloned()
        .collect()
}
