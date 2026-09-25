# Repetition inventory for metaverification

Source snapshot: `destiny-rs-dev` at
`9d062a5dd36928e754a80d9a5ffcea6c39dd4f69`.

The lexical scanner found several high-frequency proof/implementation motifs
that should not be re-proved by hand on every occurrence.

| Pattern family | Observed repetition | Reusable mechanism |
| --- | ---: | --- |
| invalid-request construction | 122 direct returns + 32 formatted returns | shared contract/error macro |
| ball-id → entity lookup | 23 exact lookups | delegation helper |
| typed component preflight | repeated Position/Rotation/Velocity/Mass lookups | generated preflight/delegation macro |
| JSON ball-id argument extraction | 14 exact sites | typed argument contract helper |
| Python operation/arity checks | 24 call checks, 14 zero-arity checks, 10 unary checks | generated operation-contract descriptors |
| repeated runtime + ball proof fixtures | 14 runtime fixtures, 13 primary-ball fixtures | proof fixture macro |
| network saturating failure accounting | 6 repeated pairs | reusable saturating counter helper |

These counts are not correctness claims.  They identify candidates for
metaprogramming.  Each candidate must have a Kani regression harness comparing
the generated/helper form to a handwritten reference form before production
code is migrated to it.

The goal is representation refactoring only: a metaprogramming migration is
acceptable only when the reference and generated forms are behaviorally
equivalent for all modeled inputs.
