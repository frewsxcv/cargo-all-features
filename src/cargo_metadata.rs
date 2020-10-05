use std::{error, path, process};

pub fn fetch() -> Result<Metadata, Box<dyn error::Error>> {
    let json = fetch_cargo_metadata_json()?;
    let json_value = json::parse(&json)?;
    Ok(Metadata::from(json_value))
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

        Dependency { name, rename, optional }
    }
}

#[derive(Clone)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub manifest_path: path::PathBuf,
    pub dependencies: Vec<Dependency>,
    pub features: Vec<String>,
}

impl From<json::JsonValue> for Package {
    fn from(json_value: json::JsonValue) -> Self {
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
            .collect();

        Package {
            id,
            name,
            manifest_path,
            dependencies,
            features,
        }
    }
}

#[derive(Clone)]
pub struct Metadata {
    pub workspace_root: path::PathBuf,
    pub workspace_members: Vec<String>,
    pub packages: Vec<Package>,
}

impl From<json::JsonValue> for Metadata {
    fn from(json_value: json::JsonValue) -> Self {
        let workspace_root =
            path::PathBuf::from(json_value["workspace_root"].as_str().unwrap().to_owned());

        let workspace_members = json_value["workspace_members"]
            .members()
            .map(|member| member.as_str().unwrap().to_owned())
            .collect();

        let packages = json_value["packages"]
            .members()
            .map(|member| Package::from(member.to_owned()))
            .collect();

        Metadata {
            workspace_root,
            workspace_members,
            packages,
        }
    }
}
