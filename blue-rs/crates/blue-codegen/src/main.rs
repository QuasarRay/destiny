use std::{collections::BTreeMap, env, fs, path::PathBuf};

use blue_codegen::{
    discover_blue_tests, discover_doxygen, discover_source_tests, write_inventory,
    write_kani_harnesses, Inventory, TestMap,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "generate".to_owned());
    if command != "generate" {
        return Err(format!("unsupported command `{command}`; expected `generate`").into());
    }

    let mut xml = PathBuf::from("target/doxygen/xml");
    let mut out = PathBuf::from("rust-port/generated");
    let mut test_map = PathBuf::from("rust-port/manifests/test-map.toml");
    let mut root = PathBuf::from(".");
    let mut deny_unmapped = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--xml" => xml = PathBuf::from(args.next().ok_or("--xml requires a path")?),
            "--out" => out = PathBuf::from(args.next().ok_or("--out requires a path")?),
            "--test-map" => {
                test_map = PathBuf::from(args.next().ok_or("--test-map requires a path")?)
            }
            "--root" => root = PathBuf::from(args.next().ok_or("--root requires a path")?),
            "--deny-unmapped" => deny_unmapped = true,
            other => return Err(format!("unknown argument `{other}`").into()),
        }
    }

    let mapping: TestMap = if test_map.exists() {
        toml::from_str(&fs::read_to_string(&test_map)?)?
    } else {
        TestMap::default()
    };
    let mappings = mapping.by_legacy();

    let (mut symbols, doxygen_tests) = discover_doxygen(&xml)?;
    let source_tests = discover_source_tests(&root)?;
    let rust_tests = discover_blue_tests(&root)?;
    let rust_test_names = rust_tests
        .iter()
        .map(|test| test.semantic_name.as_str())
        .map(ToOwned::to_owned)
        .collect::<std::collections::BTreeSet<String>>();

    let mut tests = BTreeMap::new();
    for mut test in doxygen_tests.into_iter().chain(source_tests) {
        test.rust_test = mappings
            .get(test.legacy_name.as_str())
            .map(|value| (*value).to_owned());
        tests.entry(test.legacy_name.clone()).or_insert(test);
    }

    symbols.sort();
    symbols.dedup();
    let inventory = Inventory {
        symbols,
        tests: tests.into_values().collect(),
        rust_tests,
    };
    write_inventory(&out, &inventory)?;
    write_kani_harnesses(&root, &out, &inventory.rust_tests)?;

    let unmapped = inventory
        .tests
        .iter()
        .filter(|test| test.rust_test.is_none())
        .count();
    let dangling = inventory
        .tests
        .iter()
        .filter_map(|test| test.rust_test.as_deref())
        .filter(|name| !rust_test_names.contains(*name))
        .count();
    println!(
        "blue-codegen: {} Doxygen symbols, {} legacy tests, {} mapped, {} unmapped, {} native blue tests, {} generated Kani harnesses, {} dangling mappings",
        inventory.symbols.len(),
        inventory.tests.len(),
        inventory.tests.len() - unmapped,
        unmapped,
        inventory.rust_tests.len(),
        inventory.rust_tests.len(),
        dangling,
    );

    if dangling != 0 {
        return Err(format!("{dangling} legacy test mappings target missing #[blue_test] functions").into());
    }
    if deny_unmapped && unmapped != 0 {
        return Err(format!("{unmapped} legacy tests remain unmapped").into());
    }
    Ok(())
}
