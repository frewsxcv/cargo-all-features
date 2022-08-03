use crate::metadata::Package;
use crate::Errors;
use rayon::prelude::*;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

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
