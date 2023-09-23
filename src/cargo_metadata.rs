use crate::types::{Feature, FeatureList};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::{error, path, process};

pub fn fetch() -> Result<Metadata, Box<dyn error::Error>> {
    let json = fetch_cargo_metadata_json()?;
    let json = json::parse(&json)?;
    Ok(Metadata::try_from(json)?)
}

fn fetch_cargo_metadata_json() -> Result<String, Box<dyn error::Error>> {
    let mut command = process::Command::new(crate::cargo_cmd());

    command.arg("metadata").arg("--format-version").arg("1");

    let output = command.stderr(process::Stdio::inherit()).output()?;

    if !output.status.success() {
        return Err("`cargo metadata` returned a non-zero status".into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

#[derive(Clone)]
pub struct Dependency {
    pub name: String,
    pub rename: Option<String>,
    pub optional: bool,
}

impl From<json::JsonValue> for Dependency {
    fn from(json_value: json::JsonValue) -> Self {
        let name = json_value["name"].as_str().unwrap().to_owned();
        let rename = json_value["rename"].as_str().map(|s| s.to_string());
        let optional = json_value["optional"].as_bool().unwrap();

        Dependency {
            name,
            rename,
            optional,
        }
    }
}

#[derive(Clone)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub manifest_path: path::PathBuf,
    pub dependencies: Vec<Dependency>,
    pub features: FeatureList,
    pub feature_map: HashMap<String, FeatureList>,
    pub skip_feature_sets: Vec<FeatureList>,
    pub skip_optional_dependencies: bool,
    pub allowlist: FeatureList,
    pub denylist: HashSet<Feature>,
    pub extra_features: FeatureList,
    pub always_include_features: Vec<FeatureList>,
    pub max_combination_size: Option<usize>,
}

impl TryFrom<json::JsonValue> for Package {
    type Error = String;
    fn try_from(json_value: json::JsonValue) -> Result<Self, String> {
        let id = json_value["id"].as_str().unwrap().to_owned();
        let name = json_value["name"].as_str().unwrap().to_owned();
        let manifest_path =
            path::PathBuf::from(json_value["manifest_path"].as_str().unwrap().to_owned());
        let dependencies = json_value["dependencies"]
            .members()
            .map(|member| Dependency::from(member.to_owned()))
            .collect();
        let features = json_value["features"]
            .entries()
            .map(|(k, _v)| k.to_owned())
            .map(Feature)
            .collect();
        let feature_map = json_value["features"]
            .entries()
            .map(|(k, v)| {
                (
                    k.to_owned(),
                    FeatureList(v.members().map(|v| Feature(v.to_string())).collect()),
                )
            })
            .collect();
        let skip_feature_sets: Vec<FeatureList> = json_value["metadata"]["cargo-all-features"]
            ["skip_feature_sets"]
            .members()
            .map(|member| {
                member
                    .members()
                    .map(|feature| feature.as_str().unwrap().to_owned())
                    .map(Feature)
                    .collect()
            })
            .collect();
        let maybe_skip_optional =
            json_value["metadata"]["cargo-all-features"]["skip_optional_dependencies"].as_bool();
        let skip_optional_dependencies: bool = maybe_skip_optional.unwrap_or(false);
        let extra_features: FeatureList = json_value["metadata"]["cargo-all-features"]
            ["extra_features"]
            .members()
            .map(|member| member.as_str().unwrap().to_owned())
            .map(Feature)
            .collect();

        let allowlist: FeatureList = json_value["metadata"]["cargo-all-features"]["allowlist"]
            .members()
            .map(|member| member.as_str().unwrap().to_owned())
            .map(Feature)
            .collect();

        let denylist: HashSet<_> = json_value["metadata"]["cargo-all-features"]["denylist"]
            .members()
            .map(|member| member.as_str().unwrap().to_owned())
            .map(Feature)
            .collect();
        let always_include_features: Vec<FeatureList> = json_value["metadata"]
            ["cargo-all-features"]["always_include_features"]
            .members()
            .map(|member| {
                member
                    .members()
                    .map(|feature| feature.as_str().unwrap().to_owned())
                    .map(Feature)
                    .collect()
            })
            .collect();
        let max_combination_size =
            json_value["metadata"]["cargo-all-features"]["max_combination_size"].as_usize();

        if !allowlist.is_empty() {
            if !always_include_features.is_empty() {
                return Err(format!(
                    "Package {} has both `allowlist` and `always_include_features` keys",
                    name
                ));
            }
            if !denylist.is_empty() {
                return Err(format!(
                    "Package {} has both `allowlist` and `denylist` keys",
                    name
                ));
            }
            if !extra_features.is_empty() {
                return Err(format!(
                    "Package {} has both `allowlist` and `extra_features` keys",
                    name
                ));
            }
            if maybe_skip_optional.is_some() {
                return Err(format!(
                    "Package {} has both `allowlist` and `skip_optional_dependencies` keys",
                    name
                ));
            }
            if max_combination_size.is_some() {
                return Err(format!(
                    "Package {} has both `allowlist` and `max_combination_size` keys",
                    name
                ));
            }
        }

        for list in &always_include_features {
            let one_of: HashSet<_> = list.iter().collect();
            if denylist.iter().any(|f| !one_of.contains(f)) {
                return Err(format!(
                    "Package {} has all features in a list of `always_include_features` also in `denylist`",
                    name
                ));
            }
        }

        Ok(Package {
            id,
            name,
            manifest_path,
            dependencies,
            features,
            feature_map,
            skip_feature_sets,
            skip_optional_dependencies,
            extra_features,
            allowlist,
            denylist,
            always_include_features,
            max_combination_size,
        })
    }
}

#[derive(Clone)]
pub struct Metadata {
    pub workspace_root: path::PathBuf,
    pub workspace_members: Vec<String>,
    pub packages: Vec<Package>,
}

impl TryFrom<json::JsonValue> for Metadata {
    type Error = String;
    fn try_from(json_value: json::JsonValue) -> Result<Self, String> {
        let workspace_root =
            path::PathBuf::from(json_value["workspace_root"].as_str().unwrap().to_owned());

        let workspace_members = json_value["workspace_members"]
            .members()
            .map(|member| member.as_str().unwrap().to_owned())
            .collect();

        let packages = json_value["packages"]
            .members()
            .map(|member| Package::try_from(member.to_owned()))
            .collect::<Result<_, String>>()?;

        Ok(Metadata {
            workspace_root,
            workspace_members,
            packages,
        })
    }
}
