use crate::{
    cargo_metadata::Dependency,
    rules::{new_rule, Expr},
    types::{Feature, FeatureList},
};
use itertools::Itertools;
use std::collections::HashSet;

type NamedRule = (String, Expr);

pub fn fetch_feature_sets(
    package: &crate::cargo_metadata::Package,
) -> Result<Vec<FeatureList>, String> {
    let mut features = HashSet::new();

    if package.allowlist.is_empty() {
        features.extend(fetch_optional_dependencies(package));
        features.extend(fetch_features(package));
        features.extend(package.extra_features.iter().cloned());
    } else {
        features.extend(package.allowlist.iter().cloned());
    }
    features.extend(package.always_include_features.iter().cloned());

    let mut named_rules: Vec<NamedRule> = vec![];
    for (feature, implied_features) in &package.feature_map {
        if implied_features.len() == 0 {
            continue;
        }
        let rule_name = format!("implication:{}", feature);
        let implied_features: Vec<_> = implied_features
            .iter()
            .map(|f| Feature(f.0.strip_prefix("dep:").unwrap_or(&f.0[..]).to_owned()))
            .filter(|f| features.contains(f))
            .collect();
        if let Some(rule) = new_rule::implication(
            FeatureList(vec![Feature(feature.to_owned())]).iter(),
            implied_features.iter(),
        ) {
            named_rules.push((rule_name, rule));
        }
    }
    if package.skip_optional_dependencies {
        let denied_opt_deps: Vec<_> = fetch_optional_dependencies(package)
            .filter(|dep| !package.extra_features.contains(dep))
            .collect();
        if let Some(rule) = new_rule::not_any(denied_opt_deps.iter()) {
            named_rules.push(("skip_optional_dependencies".to_owned(), rule));
        }
    }
    if let Some(rule) = new_rule::not_any(package.denylist.iter()) {
        named_rules.push(("denylist".to_owned(), rule));
    }
    if let Some(rule) = new_rule::all(package.always_include_features.iter()) {
        named_rules.push(("always_include_features".to_owned(), rule));
    }
    for conflict in package.skip_feature_sets.iter() {
        named_rules.push((
            format!(
                "conflict:{}",
                conflict.iter().map(|f| f.0.clone()).join(",")
            ),
            new_rule::not_all(conflict.iter()).unwrap(),
        ));
    }
    for rule in package.rules.iter() {
        named_rules.push((rule.to_owned(), new_rule::expr(rule)));
    }

    let max_combination_size = package.max_combination_size.unwrap_or(features.len());
    let ans = create_valid_feature_sets(&features, &named_rules, max_combination_size)?;
    if ans.is_empty() {
        Err(format!(
            "no feature set validates against the given rules: {}",
            report_greedy_rule_filtering(&features, named_rules, max_combination_size,)?
        ))
    } else {
        Ok(ans)
    }
}

fn create_valid_feature_sets(
    features: &HashSet<Feature>,
    rules: &Vec<NamedRule>,
    max_combination_size: usize,
) -> Result<Vec<FeatureList>, String> {
    let mut feature_sets = Vec::new();
    for n in 0..=max_combination_size {
        'outer: for feature_set in features.iter().combinations(n) {
            let feature_set = HashSet::from_iter(feature_set);
            for (_name, rule) in rules {
                if !rule.eval(&feature_set)? {
                    continue 'outer;
                }
            }
            feature_sets.push(feature_set.into_iter().cloned().collect());
        }
    }
    Ok(feature_sets)
}

fn report_greedy_rule_filtering(
    features: &HashSet<Feature>,
    mut rules: Vec<NamedRule>,
    max_combination_size: usize,
) -> Result<String, String> {
    let mut feature_sets = Vec::new();
    for n in 0..=max_combination_size {
        for fset in features.iter().combinations(n) {
            feature_sets.push(HashSet::from_iter(fset));
        }
    }
    let mut ans = format!("initial #sets {}", feature_sets.len());
    while !rules.is_empty() && !feature_sets.is_empty() {
        let (rule, left_overs) = largest_rule_filter(&feature_sets, &rules)?;
        if left_overs.len() == feature_sets.len() {
            break;
        }
        ans += &format!(
            " -> {} ({}/{})",
            rule.0,
            left_overs.len(),
            feature_sets.len()
        );
        let index = rules.iter().position(|x| std::ptr::eq(x, rule)).unwrap();
        rules.remove(index);
        feature_sets = left_overs;
    }
    Ok(ans)
}

fn largest_rule_filter<'a, 'b>(
    feature_sets: &Vec<HashSet<&'a Feature>>,
    rules: &'b Vec<NamedRule>,
) -> Result<(&'b NamedRule, Vec<HashSet<&'a Feature>>), String> {
    let mut ans = (&rules[0], feature_sets.clone());
    for rule in rules {
        let mut left_overs = Vec::new();
        for fset in feature_sets {
            if rule.1.eval(fset)? {
                left_overs.push(fset.clone());
            }
            if left_overs.len() >= ans.1.len() {
                break;
            }
        }
        if left_overs.len() < ans.1.len() {
            ans = (rule, left_overs);
        }
    }
    Ok(ans)
}

fn create_implicit_feat_dependency_filter(
    package: &crate::cargo_metadata::Package,
) -> impl FnMut(&Feature) -> bool + '_ {
    let mut explicit_deps: HashSet<&str> = HashSet::new();
    for (feature, implied_features) in &package.feature_map {
        if implied_features.len() == 1 && implied_features[0] == "dep:".to_owned() + feature {
            // Feature of the shape foo = ["dep:foo"]
            continue;
        } else {
            for implied_dep in implied_features
                .iter()
                .filter_map(|v| v.strip_prefix("dep:"))
            {
                explicit_deps.insert(implied_dep);
            }
        }
    }
    move |f| !explicit_deps.contains(&f[..])
}

fn fetch_optional_dependencies(
    package: &crate::cargo_metadata::Package,
) -> impl Iterator<Item = Feature> + '_ {
    package
        .dependencies
        .iter()
        .filter_map(Dependency::as_feature)
        .filter(create_implicit_feat_dependency_filter(package))
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
