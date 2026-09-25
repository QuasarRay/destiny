# Implementation status

The authoritative title-by-title ledger is
`registry/implementation_status.json`. Its schema-v2 records are regenerated
from the shipped facade and native dispatch source, not from a manually curated
coverage set.

| Status | Count | Meaning |
|---|---:|---|
| `implemented-conformant` | 2 | Tested legacy call/result contract. |
| `implemented-with-accepted-difference` | 92 | Rust handler reachable through the C ABI; declared semantic difference applies. |
| `implemented-state-only` | 2 | Reachable state, intentionally no unsupported controller effect. |
| `facade-only` | 209 | Static Python/Carbon surface, no Rust dispatch claim. |
| `unsupported` | 84 | No shipped Python object path or Rust handler. |
| **Total** | **389** | One record per audited title. |

The two conformant entries are the module and Ballpark `GetBoxCenter` call
shapes. The state-only entries are `Ball.speedFraction` and
`Ballpark.SetSpeedFraction`.

Regenerate and verify the ledger with:

```bash
python3 tools/generate_implementation_status.py
python3 -m unittest tests.test_registry -v
```

The generator resolves Python members with `inspect.getattr_static`, extracts
accepted title/member identifiers from the Rust dispatch functions, and applies
the semantic notes in `registry/implementation_annotations.json`.
