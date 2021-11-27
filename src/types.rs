//! This mod contains newtype structs [`Feature`] and [`FeatureList`], they are
//! transparent wrappers around [`String`] and [`Vec<String>`].

use std::{
    convert::{AsMut, AsRef},
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

/// A transparent wrapper around [`Vec<String>`]
#[derive(Default, Clone)]
pub struct FeatureList(pub(crate) Vec<Feature>);

/// A transparent wrapper around [`String`]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Feature(pub(crate) String);

impl FromIterator<Feature> for FeatureList {
    fn from_iter<T: IntoIterator<Item = Feature>>(iter: T) -> Self {
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

impl Deref for FeatureList {
    type Target = Vec<Feature>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for &Feature {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl Deref for Feature {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: AsRef<str>> PartialEq<S> for Feature {
    fn eq(&self, other: &S) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<str> for &Feature {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}
