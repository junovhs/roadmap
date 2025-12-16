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
roadmap add "Build API" --after "Implement Auth" --test "cargo test api_"

# See what's unproven and unblocked
roadmap next

# Start working on a claim
roadmap do "Setup Database"

# Run verification (the truth oracle)
roadmap check

# Or manually attest (for design/planning work)
roadmap check --force --reason "Design reviewed and approved"
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

### Design Principles

1. **Graph, not List:** Projects are DAGs. You cannot build the roof before the foundation.
2. **DONE is Derived:** A claim is not proven until `prove_cmd` returns Exit Code 0.
3. **Local Velocity:** Built in Rust on SQLite. <15ms cold start.
4. **Agent-First:** The CLI is "Ground Truth" for AI Agents, preventing hallucinated progress.

---

## Commands

| Command | Description |
|---------|-------------|
| `roadmap init` | Initialize `.roadmap/state.db` |
| `roadmap add <title>` | Add claim with `--after`, `--blocks`, `--test` |
| `roadmap next [--json]` | Show frontier (unblocked, unproven) |
| `roadmap do <claim>` | Set active claim (validates deps) |
| `roadmap check` | Run `prove_cmd`, store proof, mark DONE |
| `roadmap check --force --reason "..."` | Mark ATTESTED without verification |
| `roadmap list` | Show all claims |
| `roadmap status` | Overview dashboard |

---

## For AI Agents

```bash
# 1. Query the oracle
NEXT=$(roadmap next --json | jq -r '.[0].slug')

# 2. Focus
roadmap do "$NEXT" --strict

# 3. Do the work...

# 4. Verify
roadmap check
```

**Rules:**
1. Do not hallucinate claims. Run `roadmap next --json`.
2. Do not mark claims done. Run `roadmap check`.
3. Respect the graph. Blocked work stays blocked.

---

## Development Status

### v0.1.0 ? - Core Implementation
- Database engine (SQLite)
- Graph engine (petgraph, cycle detection)
- Fuzzy claim resolution
- Verification runner (shell execution)
- CLI: init, add, next, list, do, check, status

### v0.1.1 ? - Ship-Worthy
- Proof evidence capture (`{cmd, exit_code, sha, timestamp}`)
- DB hardening (foreign_keys, WAL, transactions)
- Fuzzy strict mode (`--strict` flag)
- Renamed internals: `critical_path`  `frontier`

### v0.1.2 ? - Dogfood-Ready
- `--force --reason` flag for ATTESTED state
- **Roadmap is now tracking its own development**

### v0.2.0 ?? - Derived Truth
- [ ] Computed status: UNPROVEN/PROVEN/STALE/BROKEN
- [ ] Scope field (what files invalidate a proof)
- [ ] `roadmap stale` command

### v0.3.0 - Audit & History
- [ ] Append-only proof history
- [ ] `roadmap why <claim>` - show proof chain

---

## Building

```bash
cargo build --release
cargo install --path .
```

---

*"What is true, right now, in this repo?"*