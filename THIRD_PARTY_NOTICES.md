# Third-party notices

This compatibility implementation is derived from the open-source Destiny
code released by CCP Games under the MIT License. The preserved notice is in
`LICENSE`.

Direct native dependencies are pinned in `Cargo.toml` and `Cargo.lock`:

| Component | Version requested | Upstream license expression |
|---|---:|---|
| Avian (`avian3d`) | 0.7.0 | MIT OR Apache-2.0 |
| Bevy | 0.19.1 | MIT OR Apache-2.0 |
| `bevy_replicon` | 0.42.3 | MIT OR Apache-2.0 |
| Lightyear component crates | 0.29.0 | MIT OR Apache-2.0 |
| `base64` | 0.22.1 | MIT OR Apache-2.0 |
| `serde` | 1.0.228 | MIT OR Apache-2.0 |
| `serde_json` | 1.0.150 | MIT OR Apache-2.0 |
| `thiserror` | 2.0.18 | MIT OR Apache-2.0 |

This table is informational and is not a substitute for a generated inventory
of every transitive binary dependency. `cargo deny check` and review of the
final wheel/binary notice bundle are mandatory release gates. Do not represent
this source notice as completed legal review.
