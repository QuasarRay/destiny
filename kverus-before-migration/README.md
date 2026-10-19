# KVerus pre-migration snapshot

This directory is a byte-identical archival copy of the reusable KVerus implementation at `f62f1d79304596098be917d2524702d926b69be5` on `KVerus-dev`, captured immediately before the CGP / proc-macro2 / arcane-rs refactor.

It intentionally does **not** copy the parent Destiny runtime or Destiny-specific proof corpus. The exact inclusion boundary and original Git blob SHA for every copied file are recorded in `SNAPSHOT.json`.

The live implementation remains in its original `verification/...` locations; this directory is archival and must not be edited as part of the migration.
