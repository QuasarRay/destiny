use blue::prelude::*;

#[blue_test]
fn runtime_enters_running_state_on_first_update() {
    let mut runtime = BlueRuntime::new();
    assert_eq!(
        runtime.world().resource::<BlueLifecycle>().state(),
        BlueLifecycleState::Starting
    );

    runtime.update();

    assert_eq!(
        runtime.world().resource::<BlueLifecycle>().state(),
        BlueLifecycleState::Running
    );
    assert_eq!(runtime.world().resource::<BlueClock>().frame(), 1);
}

#[blue_test]
fn startup_args_preserve_carbon_has_semantics() {
    let mut runtime = BlueRuntime::new();
    runtime.set_startup_args(["/packaged".to_owned(), "/foo".to_owned()]);

    let args = runtime.world().resource::<BlueStartupArgs>();
    assert!(args.contains("/packaged"));
    assert!(!args.contains("/missing"));
    assert_eq!(args.count(), 2);
}

#[blue_test]
fn resource_index_uses_bevy_assets_as_storage() {
    let mut runtime = BlueRuntime::new();
    let handle = runtime.add_resource("res:/hello", b"blue".to_vec());

    let index = runtime.world().resource::<BlueResourceIndex>();
    assert_eq!(index.get("res:/hello"), Some(&handle));

    let assets = runtime.world().resource::<bevy::prelude::Assets<BlueResource>>();
    let asset = assets.get(&handle).expect("resource asset must exist");
    assert_eq!(asset.bytes, b"blue");
}

#[blue_test]
fn search_paths_match_blue_set_get_and_clear_semantics() {
    let mut paths = BluePaths::default();
    assert_eq!(paths.get_search_path("test"), "");

    paths
        .set_search_path("test", "some/value")
        .expect("ordinary search path must be accepted");
    assert_eq!(paths.get_search_path("TEST"), "some/value");
    assert_eq!(paths.all_search_paths().len(), 1);

    paths
        .set_search_path("test", "")
        .expect("empty search path removes the mapping");
    assert_eq!(paths.get_search_path("test"), "");
    assert!(paths.all_search_paths().is_empty());
}

#[blue_test]
fn circular_search_paths_are_rejected_without_committing_candidate() {
    let mut paths = BluePaths::default();
    assert!(paths.set_search_path("selfRef1", "selfRef1:").is_err());
    assert_eq!(paths.get_search_path("selfRef1"), "");

    paths
        .set_search_path("selfRef1", "selfRef2:")
        .expect("forward reference without a cycle is allowed");
    assert!(paths.set_search_path("selfRef2", "selfRef1:").is_err());
    assert_eq!(paths.get_search_path("selfRef2"), "");
}
