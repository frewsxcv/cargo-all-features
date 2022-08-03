use crate::types::FeatureList;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Clone, Deserialize, Debug)]
pub struct CargoAllFeatures {
    pub skip_feature_sets: Option<Vec<FeatureList>>,
    pub skip_optional_dependencies: Option<bool>,
    pub extra_features: Option<FeatureList>,
    pub allowlist: Option<FeatureList>,
    pub denylist: Option<HashSet<String>>,
}
