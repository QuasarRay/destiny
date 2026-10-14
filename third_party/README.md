# Vendored verification upstreams

Destiny pins the verifier source trees as Git submodules rather than downloading
floating release archives during normal development.

- `third_party/verus`: Verus release `0.2026.09.20.aef82ed`, commit
  `aef82eda71838deef4a8cd0260fda6c9504be9b9`.
- `third_party/kani`: Kani `0.68.0`, commit
  `0d2328a93f0e0ff66132d6bfa1a7d884877cf862`.

Kani itself contains three pinned submodules at this revision. Always materialize
recursively. The authoritative machine-readable provenance is
`verification/vendor-lock.json`.

Use:

```sh
python3 -B verification/tools/bootstrap_verifiers.py --init
python3 -B verification/tools/bootstrap_verifiers.py --check
```

The two verifiers intentionally remain separate build processes. Verus pins Rust
1.98.1 while Kani pins nightly-2026-08-21 and rustc-private components; forcing
their compiler internals into one Cargo workspace would weaken reproducibility.

Vendoring does not make Kani/CBMC part of the default Verus proof engine. Kani
source may supply explicitly identified reusable discovery/metadata/test
components. Residual Kani proof obligations remain visible until an equivalent
or stronger Verus-backed obligation has been demonstrated.
