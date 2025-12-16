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

### The Claim Model

Everything in Roadmap is a **Claim** - a statement about your project that can be proven.

```
Claim {
    statement: "Auth rejects invalid credentials"
    prove_cmd: "cargo test auth::test_invalid_login"
    depends_on: [setup-database]
}
```

**Derived States:**
- `UNPROVEN` - no proof exists
- `PROVEN` - verifier passed, proof is current
- `STALE` - proof passed, but relevant code changed since
- `BROKEN` - verifier ran and failed

### Design Principles

1. **Graph, not List:** Projects are DAGs. You cannot build the roof before the foundation.
2. **DONE is Derived:** A claim is not proven until `prove_cmd` returns Exit Code 0.
3. **Local Velocity:** Built in Rust on SQLite. <15ms cold start.
4. **Agent-First:** The CLI is "Ground Truth" for AI Agents, preventing hallucinated progress.

---

## Commands

### `roadmap init`
Initialize `.roadmap/state.db` in the current directory.

### `roadmap add <title>`
Add a new claim with optional dependencies and verification.

```bash
# Simple claim
roadmap add "Write documentation"

# With proof command (the oracle)
roadmap add "Fix auth bug" --test "cargo test auth_middleware"

# With dependency (must prove "Auth" first)
roadmap add "Build settings page" --after "Auth"

# This claim blocks another
roadmap add "Design system" --blocks "Build UI"
```

**Fuzzy Resolution:** Claim references can be:
- Exact ID: `42`
- Exact slug: `setup-database`
- Fuzzy match: `"database"`, `"auth"`, `"Setup"`

### `roadmap next`
Show the **frontier** - claims that are unproven and unblocked.

```bash
# Human-readable
roadmap next

# Agent-friendly JSON (strict mode, no fuzzy guessing)
roadmap next --json
```

### `roadmap do <claim>`
Set focus to a claim. Validates dependencies are satisfied.

```bash
roadmap do "auth"
# ? Now working on: [implement-auth] Implement Authentication
```

### `roadmap check`
**The Verification Oracle.** Runs `prove_cmd` for the active claim.

- **Exit 0:** Claim marked `PROVEN`, proof evidence stored, children unblocked.
- **Exit ? 0:** Claim remains `UNPROVEN`, stderr displayed.

```bash
roadmap check
# ?? Checking: [implement-auth] Implement Authentication
#    running: cargo test auth_
# � Verified! Claim [implement-auth] marked PROVEN (0.42s)
```

### `roadmap list`
Show all claims with their status.

### `roadmap status`
Overview: proven count, active claim, next available.

---

## For AI Agents

If you are an AI Agent reading this:

1. **Do not hallucinate claims.** Run `roadmap next --json` to see what is actually required.
2. **Do not mark claims proven.** Run `roadmap check` and let the exit code decide.
3. **Respect the Graph.** You cannot work on a claim if its dependencies are not proven.

### Agent Loop

```bash
# 1. Query the oracle
NEXT=$(roadmap next --json | jq -r '.[0].slug')

# 2. Focus
roadmap do "$NEXT"

# 3. Do the work...
# (your code changes here)

# 4. Verify
roadmap check
# If it passes, loop. If it fails, fix and retry.
```

---

## Architecture

### Data Layer (`.roadmap/state.db`)
SQLite with ACID guarantees. WAL mode, foreign keys enforced.

**Schema:**
- `claims`: Nodes (id, slug, statement, prove_cmd, proof_json)
- `dependencies`: Edges (blocker_id  blocked_id)
- `state`: Key-value for current context

### Graph Engine (`petgraph`)
- Frontier calculation (unblocked, unproven)
- Cycle detection at insertion time
- O(V+E) traversal

### Verification Runner
Tool-agnostic. Only understands:
- Shell commands
- Exit codes (0 = proven)
- Evidence capture (sha, timestamp, duration)

---

## Development Roadmap

### v0.1.0 ? - Core Implementation
- [x] Database engine (SQLite)
- [x] Graph engine (petgraph, cycle detection)
- [x] Fuzzy claim resolution
- [x] Verification runner (shell execution)
- [x] CLI: init, add, next, list, do, check, status

### v0.1.1 ?? - Ship-Worthy
- [ ] Proof evidence capture (`{cmd, exit_code, sha, timestamp}`)
- [ ] DB hardening (foreign_keys, WAL, transactions)
- [ ] Fuzzy strict mode (no guessing in `--json`)
- [ ] Rename internals: `critical_path`  `frontier`

### v0.2.0 - Derived Truth
- [ ] Computed status: UNPROVEN/PROVEN/STALE/BROKEN
- [ ] Scope field (what files invalidate a proof)
- [ ] `roadmap stale` command
- [ ] Rename: Task  Claim

### v0.3.0 - Attestation & Audit
- [ ] ATTESTED state (`--force` with reason, not PROVEN)
- [ ] Append-only proof history
- [ ] `roadmap why <claim>` - show proof chain

---

## Building

```bash
cargo build --release
# Binary at: target/release/roadmap

# Or install globally
cargo install --path .
```

---

*"What is true, right now, in this repo?"*