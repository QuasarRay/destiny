use std::collections::BTreeMap;

use bevy::{asset::Asset, prelude::*};
use delegate::delegate;

#[derive(Asset, Clone, Debug, Default, Reflect)]
pub struct BlueResource {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl BlueResource {
    #[must_use]
    pub fn new(name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            bytes: bytes.into(),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct BlueResourceIndex {
    resources: BTreeMap<String, Handle<BlueResource>>,
}

impl BlueResourceIndex {
    delegate! {
        to self.resources {
            pub fn len(&self) -> usize;
            pub fn is_empty(&self) -> bool;
            pub fn clear(&mut self);
        }
    }

    pub fn insert(&mut self, name: impl Into<String>, handle: Handle<BlueResource>) {
        self.resources.insert(name.into(), handle);
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Handle<BlueResource>> {
        self.resources.get(name)
    }

    #[must_use]
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.resources.keys().map(String::as_str)
    }
}
