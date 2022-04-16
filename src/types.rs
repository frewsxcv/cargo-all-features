/// This mod contains newtype structs [`Feature`] and [`FeatureList`], they are
/// transparent wrappers around [`String`] and [`Vec<String>`].
use std::{
    convert::{AsMut, AsRef},
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use serde::Deserialize;

/// A transparent wrapper around [`Vec<String>`]
#[derive(Default, Clone, Debug, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct FeatureList<T = String>(pub(crate) Vec<T>);

impl FromIterator<String> for FeatureList {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        FeatureList(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a String> for FeatureList<&'a String> {
    fn from_iter<T: IntoIterator<Item = &'a String>>(iter: T) -> Self {
        FeatureList(iter.into_iter().collect())
    }
}

impl AsMut<<FeatureList as Deref>::Target> for &mut FeatureList {
    fn as_mut(&mut self) -> &mut <FeatureList as Deref>::Target {
        self.deref_mut()
    }
}

impl AsRef<<FeatureList as Deref>::Target> for &FeatureList {
    fn as_ref(&self) -> &<FeatureList as Deref>::Target {
        self.deref()
    }
}

impl DerefMut for FeatureList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> DerefMut for FeatureList<&'a String> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> DerefMut for FeatureList<&'a &'a String> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Deref for FeatureList<&'a &'a String> {
    type Target = Vec<&'a &'a String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Deref for FeatureList<&'a String> {
    type Target = Vec<&'a String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for FeatureList {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
