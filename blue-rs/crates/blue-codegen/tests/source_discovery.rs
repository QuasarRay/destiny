use blue_codegen::{discover_gtest_text, discover_python_text};
use blue_macros::blue_test;

#[blue_test]
fn python_discovery_preserves_class_scope() {
    let tests = discover_python_text(
        "tests/python/test_sample.py",
        "class Case:\n    def testWorks(self):\n        pass\n",
    );
    assert_eq!(tests.len(), 1);
    assert!(tests[0].legacy_name.contains("Case::testWorks"));
}

#[blue_test]
fn gtest_discovery_preserves_fixture_and_case() {
    let tests = discover_gtest_text(
        "tests/cpp/TestSample.cpp",
        "TEST_F(Sample, Works) { }\n",
    );
    assert_eq!(tests.len(), 1);
    assert!(tests[0].legacy_name.contains("Sample::Works"));
}
