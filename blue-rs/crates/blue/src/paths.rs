use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use blueexposure::{blue_methods, blue_pyclass};
use thiserror::Error;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum BluePathError {
    #[error("search paths contain a circular reference")]
    CircularReference,
}

#[blue_pyclass(module = "blue", name = "BluePaths", kind = "resource")]
#[derive(Clone, Debug)]
pub struct BluePaths {
    #[blue(skip)]
    #[reflect(ignore)]
    search_paths: BTreeMap<String, String>,
    #[blue(read, write)]
    packaged: bool,
    #[blue(read, write)]
    initial_working_directory: String,
}

impl Default for BluePaths {
    fn default() -> Self {
        Self {
            search_paths: BTreeMap::new(),
            packaged: false,
            initial_working_directory: ".".to_owned(),
        }
    }
}

impl BluePaths {
    pub fn set_search_path(
        &mut self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<(), BluePathError> {
        let key = key.as_ref().to_ascii_lowercase();
        let value = value.as_ref().replace('\\', "/");

        if value.is_empty() {
            self.search_paths.remove(&key);
            return Ok(());
        }

        let mut candidate = self.search_paths.clone();
        candidate.insert(key.clone(), value.clone());
        let mut active = BTreeSet::new();
        if expand_value(&value, &candidate, &mut active).is_none() {
            return Err(BluePathError::CircularReference);
        }

        self.search_paths.insert(key, value);
        Ok(())
    }

    #[must_use]
    pub fn get_search_path(&self, key: &str) -> &str {
        self.search_paths
            .get(&key.to_ascii_lowercase())
            .map(String::as_str)
            .unwrap_or("")
    }

    #[must_use]
    pub fn all_search_paths(&self) -> BTreeMap<String, String> {
        self.search_paths.clone()
    }

    pub fn clear_search_paths(&mut self) {
        self.search_paths.clear();
    }

    #[must_use]
    pub fn expanded_search_paths(&self, key: &str) -> Vec<String> {
        let key = key.to_ascii_lowercase();
        let Some(value) = self.search_paths.get(&key) else {
            return Vec::new();
        };
        expand_value(value, &self.search_paths, &mut BTreeSet::from([key])).unwrap_or_default()
    }

    #[must_use]
    pub fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        let raw = path.as_ref().to_string_lossy().replace('\\', "/");
        let candidates = self.expand_runtime_path(&raw);

        if let Some(existing) = candidates.iter().find(|candidate| Path::new(candidate).exists()) {
            return absolute_from(&self.initial_working_directory, existing);
        }

        candidates
            .first()
            .map(|candidate| absolute_from(&self.initial_working_directory, candidate))
            .unwrap_or_else(|| absolute_from(&self.initial_working_directory, &raw))
    }

    #[must_use]
    pub fn resolve_for_writing(&self, path: impl AsRef<Path>) -> PathBuf {
        let raw = path.as_ref().to_string_lossy().replace('\\', "/");
        self.expand_runtime_path(&raw)
            .first()
            .map(|candidate| absolute_from(&self.initial_working_directory, candidate))
            .unwrap_or_else(|| absolute_from(&self.initial_working_directory, &raw))
    }

    #[must_use]
    fn expand_runtime_path(&self, raw: &str) -> Vec<String> {
        let Some((key, rest)) = split_prefix(raw) else {
            return vec![raw.to_owned()];
        };
        let key = key.to_ascii_lowercase();
        let Some(value) = self.search_paths.get(&key) else {
            return vec![raw.to_owned()];
        };
        let mut active = BTreeSet::from([key]);
        let bases = expand_value(value, &self.search_paths, &mut active).unwrap_or_default();
        bases
            .into_iter()
            .filter(|base| !base.is_empty())
            .map(|base| join_blue_path(&base, rest))
            .collect()
    }
}

#[blue_methods]
impl BluePaths {
    #[blue(new)]
    fn py_new(initial_working_directory: Option<String>, packaged: Option<bool>) -> Self {
        Self {
            initial_working_directory: initial_working_directory.unwrap_or_else(|| ".".to_owned()),
            packaged: packaged.unwrap_or(false),
            ..Self::default()
        }
    }

    #[blue(name = "SetSearchPath")]
    fn py_set_search_path(&mut self, key: &str, value: &str) -> pyo3::PyResult<()> {
        self.set_search_path(key, value)
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    }

    #[blue(name = "GetSearchPath")]
    fn py_get_search_path(&self, key: &str) -> String {
        self.get_search_path(key).to_owned()
    }

    #[blue(name = "GetAllSearchPaths")]
    fn py_get_all_search_paths(&self) -> BTreeMap<String, String> {
        self.all_search_paths()
    }

    #[blue(name = "ClearSearchPaths")]
    fn py_clear_search_paths(&mut self) {
        self.clear_search_paths();
    }

    #[blue(name = "ResolvePath")]
    fn py_resolve_path(&self, path: &str) -> String {
        self.resolve(path).to_string_lossy().replace('\\', "/")
    }

    #[blue(name = "ResolvePathForWriting")]
    fn py_resolve_path_for_writing(&self, path: &str) -> String {
        self.resolve_for_writing(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

fn expand_value(
    value: &str,
    search_paths: &BTreeMap<String, String>,
    active: &mut BTreeSet<String>,
) -> Option<Vec<String>> {
    let mut result = Vec::new();
    for component in value.split(';') {
        let Some((key, rest)) = split_prefix(component) else {
            result.push(component.to_owned());
            continue;
        };
        let key = key.to_ascii_lowercase();
        let Some(next) = search_paths.get(&key) else {
            continue;
        };
        if !active.insert(key.clone()) {
            return None;
        }
        let expanded = expand_value(next, search_paths, active)?;
        active.remove(&key);
        result.extend(
            expanded
                .into_iter()
                .filter(|base| !base.is_empty())
                .map(|base| join_blue_path(&base, rest)),
        );
    }
    Some(result)
}

fn split_prefix(path: &str) -> Option<(&str, &str)> {
    let pos = path.find(':')?;
    // Preserve Windows-style drive letters, matching the original `pos <= 1` rule.
    if pos <= 1 {
        return None;
    }
    Some((&path[..pos], &path[pos + 1..]))
}

fn join_blue_path(base: &str, rest: &str) -> String {
    if rest.is_empty() {
        return base.to_owned();
    }
    if rest.starts_with('/') || base.ends_with('/') {
        format!("{base}{rest}")
    } else {
        format!("{base}/{rest}")
    }
}

fn absolute_from(initial_working_directory: &str, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(initial_working_directory).join(path)
    }
}
