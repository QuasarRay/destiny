// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// @generated; do not edit. Original Destiny Copyright CCP ehf.; MIT.
// Source SHA-256: 7d74c8349232c6b31f01f3bbac0f5de2a0fc387c98dfa8449fc71b6035b96238
// Original revisions: ef738406b61ad06bae2f402857e464735ab106a2, 114e89fa584f0e1a81eb0a679aabf41af8870072
// Symbols: Vector3d::Length
// Extractor SHA-256: 5274e140b7aa210ea5eee6b443be65d4190559061ee60d6b25bb4dbd611e17fb
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791
// Configuration SHA-256: 6be0feb2d79837bd64475c9b40b1bad54ef0dca398d595f430d763da3178732e
// Rule: scalar-expression-v1
// Regenerate: python tools/migrate_destiny.py generate
pub(crate) fn vector_length(vector: [f64; 3]) -> f64 {
    #[cfg(kani)]
    kani::cover!(true, "actual generated geometry adapter executed");
    destiny_original_spec::Vec3 {
        x: vector[0],
        y: vector[1],
        z: vector[2],
    }
    .length()
}
