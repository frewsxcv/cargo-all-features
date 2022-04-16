use serde::Deserialize;

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
