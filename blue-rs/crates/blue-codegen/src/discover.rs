use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use roxmltree::Document;
use walkdir::WalkDir;

use crate::{LegacySymbol, LegacyTest, RustTest};

pub fn discover_doxygen(xml_dir: &Path) -> Result<(Vec<LegacySymbol>, Vec<LegacyTest>), std::io::Error> {
    let mut symbols = BTreeSet::new();
    let mut tests = BTreeSet::new();

    if !xml_dir.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    for entry in WalkDir::new(xml_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "xml"))
    {
        let text = fs::read_to_string(entry.path())?;
        let Ok(document) = Document::parse(&text) else {
            continue;
        };

        for compound in document.descendants().filter(|node| node.has_tag_name("compounddef")) {
            let compound_name = child_text(compound, "compoundname").unwrap_or_default();
            let kind = compound.attribute("kind").unwrap_or("compound").to_owned();
            let location = compound
                .descendants()
                .find(|node| node.has_tag_name("location"))
                .and_then(|node| node.attribute("file"))
                .unwrap_or_default()
                .to_owned();
            if !compound_name.is_empty() {
                symbols.insert(LegacySymbol {
                    source: location.clone(),
                    qualified_name: compound_name.clone(),
                    kind,
                });
            }

            for member in compound.descendants().filter(|node| node.has_tag_name("memberdef")) {
                let member_kind = member.attribute("kind").unwrap_or("member").to_owned();
                let Some(name) = child_text(member, "name") else {
                    continue;
                };
                let member_location = member
                    .descendants()
                    .find(|node| node.has_tag_name("location"));
                let source = member_location
                    .and_then(|node| node.attribute("file"))
                    .unwrap_or(&location)
                    .to_owned();
                let line = member_location
                    .and_then(|node| node.attribute("line"))
                    .and_then(|line| line.parse::<u32>().ok());
                let qualified_name = if compound_name.is_empty() {
                    name.clone()
                } else {
                    format!("{compound_name}::{name}")
                };
                symbols.insert(LegacySymbol {
                    source: source.clone(),
                    qualified_name: qualified_name.clone(),
                    kind: member_kind,
                });

                if is_test_source(&source) && is_test_name(&name) {
                    tests.insert(LegacyTest {
                        source: source.clone(),
                        legacy_name: format!("{}::{qualified_name}", normalize_path(&source)),
                        line,
                        discovery: "doxygen".to_owned(),
                        rust_test: None,
                    });
                }
            }
        }
    }

    Ok((symbols.into_iter().collect(), tests.into_iter().collect()))
}


pub fn discover_blue_tests(root: &Path) -> Result<Vec<RustTest>, std::io::Error> {
    let crates_dir = root.join("crates");
    let mut tests = BTreeSet::new();
    if !crates_dir.exists() {
        return Ok(Vec::new());
    }

    for crate_entry in fs::read_dir(&crates_dir)? {
        let crate_entry = crate_entry?;
        if !crate_entry.file_type()?.is_dir() {
            continue;
        }
        let crate_name = crate_entry.file_name().to_string_lossy().to_string();
        let tests_dir = crate_entry.path().join("tests");
        if !tests_dir.exists() {
            continue;
        }

        for entry in WalkDir::new(&tests_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
            .filter(|entry| entry.file_name().to_string_lossy() != "blue_kani_generated.rs")
        {
            let source = fs::read_to_string(entry.path())?;
            let file = syn::parse_file(&source).map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to parse {}: {error}", entry.path().display()),
                )
            })?;
            let relative_source = entry
                .path()
                .strip_prefix(&tests_dir)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/");
            let file_module = entry
                .path()
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("tests")
                .to_owned();
            collect_blue_tests(
                &crate_name,
                &relative_source,
                &[file_module],
                &file.items,
                &mut tests,
            );
        }
    }

    Ok(tests.into_iter().collect())
}

fn collect_blue_tests(
    crate_name: &str,
    source: &str,
    module_path: &[String],
    items: &[syn::Item],
    out: &mut BTreeSet<RustTest>,
) {
    for item in items {
        match item {
            syn::Item::Fn(function)
                if function
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("blue_test")) =>
            {
                let function_name = function.sig.ident.to_string();
                let semantic_name = format!(
                    "{}::{}::{}",
                    crate_name,
                    module_path.join("::"),
                    function_name
                );
                out.insert(RustTest {
                    crate_name: crate_name.to_owned(),
                    source: source.to_owned(),
                    module_path: module_path.to_vec(),
                    function: function_name.clone(),
                    semantic_name,
                    body_name: format!("__blue_test_body_{function_name}"),
                });
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested_items)) = &module.content {
                    let mut nested_path = module_path.to_vec();
                    nested_path.push(module.ident.to_string());
                    collect_blue_tests(crate_name, source, &nested_path, nested_items, out);
                }
            }
            _ => {}
        }
    }
}

pub fn discover_source_tests(root: &Path) -> Result<Vec<LegacyTest>, std::io::Error> {
    let mut tests = BTreeSet::new();
    for relative_root in ["tests", "blueexposure/tests"] {
        let base = root.join(relative_root);
        if !base.exists() {
            continue;
        }
        for entry in WalkDir::new(&base)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            match entry.path().extension().and_then(|ext| ext.to_str()) {
                Some("py") => discover_python_file(root, entry.path(), &mut tests)?,
                Some("cpp" | "cc" | "cxx") => discover_gtest_file(root, entry.path(), &mut tests)?,
                _ => {}
            }
        }
    }
    Ok(tests.into_iter().collect())
}

fn discover_python_file(
    root: &Path,
    path: &Path,
    out: &mut BTreeSet<LegacyTest>,
) -> Result<(), std::io::Error> {
    let source = fs::read_to_string(path)?;
    out.extend(discover_python_text(&relative(root, path), &source));
    Ok(())
}

fn discover_gtest_file(
    root: &Path,
    path: &Path,
    out: &mut BTreeSet<LegacyTest>,
) -> Result<(), std::io::Error> {
    let source = fs::read_to_string(path)?;
    out.extend(discover_gtest_text(&relative(root, path), &source));
    Ok(())
}

#[must_use]
pub fn discover_python_text(relative: &str, source: &str) -> Vec<LegacyTest> {
    let mut out = BTreeSet::new();
    let mut class_stack: Vec<(usize, String)> = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len().saturating_sub(trimmed.len());

        while class_stack.last().is_some_and(|(class_indent, _)| indent <= *class_indent) {
            class_stack.pop();
        }

        if let Some(rest) = trimmed.strip_prefix("class ") {
            if let Some(name) = rest.split(['(', ':']).next().map(str::trim).filter(|name| !name.is_empty()) {
                class_stack.push((indent, name.to_owned()));
            }
            continue;
        }

        let Some(rest) = trimmed.strip_prefix("def ") else {
            continue;
        };
        let Some(name) = rest.split('(').next().map(str::trim) else {
            continue;
        };
        if !is_test_name(name) {
            continue;
        }

        let scope = class_stack
            .last()
            .map(|(_, class_name)| format!("{class_name}::"))
            .unwrap_or_default();
        out.insert(LegacyTest {
            source: relative.to_owned(),
            legacy_name: format!("{relative}::{scope}{name}"),
            line: u32::try_from(index + 1).ok(),
            discovery: "source-python".to_owned(),
            rust_test: None,
        });
    }
    out.into_iter().collect()
}

#[must_use]
pub fn discover_gtest_text(relative: &str, source: &str) -> Vec<LegacyTest> {
    let mut out = BTreeSet::new();
    for (index, line) in source.lines().enumerate() {
        let compact = line.trim();
        let Some((kind, rest)) = ["TEST_F(", "TEST_P(", "TEST("]
            .into_iter()
            .find_map(|prefix| compact.find(prefix).map(|offset| (prefix, &compact[offset + prefix.len()..])))
        else {
            continue;
        };
        let Some(args) = rest.split(')').next() else {
            continue;
        };
        let mut parts = args.split(',').map(str::trim);
        let (Some(suite), Some(name)) = (parts.next(), parts.next()) else {
            continue;
        };
        out.insert(LegacyTest {
            source: relative.to_owned(),
            legacy_name: format!("{relative}::{suite}::{name}"),
            line: u32::try_from(index + 1).ok(),
            discovery: format!("source-{}", kind.trim_end_matches('(').to_ascii_lowercase()),
            rust_test: None,
        });
    }
    out.into_iter().collect()
}

fn child_text(node: roxmltree::Node<'_, '_>, tag: &str) -> Option<String> {
    node.children()
        .find(|child| child.has_tag_name(tag))
        .and_then(|child| child.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn is_test_name(name: &str) -> bool {
    name.to_ascii_lowercase().starts_with("test")
}

fn is_test_source(source: &str) -> bool {
    let normalized = normalize_path(source);
    normalized.contains("/tests/") || normalized.starts_with("tests/") || normalized.contains("/test/")
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[allow(dead_code)]
fn _canonicalish(path: &Path) -> PathBuf {
    path.components().collect()
}
