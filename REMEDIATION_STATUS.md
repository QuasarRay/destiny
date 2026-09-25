# Remediation status (v4)

`V4_REMEDIATION_STATUS.md` is the authoritative ledger for the supplied v3
audit and this independent v4 review. The supplied audit contained broad
recommendations but no concrete defect list; the v4 review found and addressed
64 specific issues across snapshots, physics, validation, FFI, C++, Carbon,
client ticker, and server batching.

The historical `V3_REMEDIATION_STATUS.md` remains unchanged for traceability to
the earlier 67-item v2 audit.

Portable Python and strict-header/C++ checks run locally. Rust, linked ABI,
packaging, supply-chain, and two-process gates remain explicitly open because
their toolchains or deployment environment are unavailable here. No open gate
is counted as passing; see `VERIFICATION.md` and `RELEASE_GATES.md`.
