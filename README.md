# Roadmap (Project Cortex)

> **A proof-carrying roadmap.**
> A DAG of claims whose completion is operationally trustworthy.

---

## Quick Start

```bash
# Initialize in your project
roadmap init

# Add claims with dependencies and proofs
roadmap add "Setup Database" --test "cargo test db_"
roadmap add "Implement Auth" --after "Setup Database" --test "cargo test auth_"

# Add a scoped claim (Smart Decay)
# This task only goes stale if files in src/api/ change
roadmap add "Build API" \
    --after "Implement Auth" \
    --test "cargo test api_" \
    --scope "src/api/**"

# See what's unproven and unblocked
roadmap next

# Start working on a claim
roadmap do "Setup Database"

# Run verification (the truth oracle)
roadmap check
```

---

## Philosophy

Most project management tools answer: *"What should we do next?"*

**Roadmap** answers: *"What is true right now, and what truth is missing?"*

### The Core Difference

| Tool | Model | "Done" means... |
|------|-------|-----------------|
| JIRA, Trello, TODO.md | Logger | "Someone said so" |
| **Roadmap** | State Machine | "The verifier passed" |

### Derived Truth & Smart Decay

Roadmap does not trust your memory. It trusts the **Proof Audit Log**.

1.  **PROVEN:** The verification command passed on the *current* Git commit.
2.  **STALE:** The verification passed previously, but the code has changed since then.
    *   **Global Decay:** By default, *any* commit invalidates proofs.
    *   **Smart Decay:** If you define a `--scope`, the proof stays valid unless the changes touch matched files.
3.  **BROKEN:** The verification command failed.

---

## Commands

| Command | Description |
|---------|-------------|
| `roadmap init` | Initialize `.roadmap/state.db` |
| `roadmap add` | Add claim with `--after`, `--blocks`, `--test`, `--scope` |
| `roadmap next` | Show frontier (unblocked, unproven) |
| `roadmap do` | Set active claim (validates deps) |
| `roadmap check` | Run `prove_cmd`, store proof, update status |
| `roadmap why` | **NEW:** Explain why a task is Stale/Proven + Audit Log |
| `roadmap stale` | **NEW:** Scan for invalidated proofs |
| `roadmap history` | **NEW:** Stream chronological verification events |
| `roadmap status` | Overview dashboard |

---

## Development Status

### v0.1.0 ✅ - Core Implementation
- Database engine (SQLite)
- Graph engine (petgraph, cycle detection)

### v0.2.0 ✅ - Derived Truth
- Audit Log (append-only proofs)
- SHA-based Global Staleness
- `why`, `stale`, and `history` commands

### v0.3.0 ✅ - Contextual Intelligence
- **Smart Decay**: Scoped invalidation using `git diff`
- **RepoContext**: Context-aware status derivation
- **`--scope`**: File pattern matching for tasks

---

*"What is true, right now, in this repo?"*