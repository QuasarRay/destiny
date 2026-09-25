# Known original-Destiny lifecycle gaps

These original tests are **not** marked proved merely because the broader
lifecycle family has a formal model.

| Original test | Current status | Reason |
| --- | --- | --- |
| `test_add_ball_creates_a_client_ball` | ProofMissing | Requires Python facade type-identity equivalence, not only Rust entity creation. |
| `test_remove_ball_no_delay` | KnownMismatch | Original retained Ball references expose `isMoribund=True`; the Rust core despawns immediately and the compatibility registry lists `destiny.Ball.isMoribund` as having no equivalent. |
| `test_remove_ball_with_delay` | ProofMissing | Rust has `DestinyPendingRemoval`, but the public retained-object `isMoribund` observation still needs a facade adapter proof. |
| `test_clear_all_removes_ball_from_park` | ImplementationMissing | The original assertion observes `GetBubbleAtCoordinates`; the current Rust dispatch does not expose that original operation. |
| mushroom lifecycle tests | ImplementationMissing | No equivalent original mushroom state machine is implemented in the current Rust subset. |
| negative generated ball/capsule/box ID tests | ImplementationMissing | The current Rust AddBall path treats supplied IDs literally and does not implement the original shared negative-ID generator. |

These remain visible obligations in the exhaustive catalog and are intentionally
absent from paired proof markers until their concrete semantics exist.
