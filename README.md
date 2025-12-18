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
# NOTE: Repository must be clean (committed) to verify!
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
| **Roadmap** | State Machine | "The verifier passed on this Commit" |

### 1. Derived Truth
Roadmap does not trust your memory. It trusts the **Proof Audit Log**.
- **PROVEN:** The verification passed on the *current* Git commit.
- **STALE:** The verification passed previously, but the code has changed since then.
- **BROKEN:** The verification command failed.

### 2. Smart Decay (Contextual Intelligence)
Codebases change. Roadmap knows exactly *what* changed.
- **Global Decay:** If a task has no scope, *any* commit marks it STALE. (Safe default).
- **Smart Decay:** If a task has a `--scope` (e.g., `src/auth/**`), it stays PROVEN unless the changes touch those files.

### 3. The Law of Hygiene (Strict Mode)
Truth is a property of a Commit, not a Worktree.
- Roadmap **refuses** to run `check` if your repository is dirty.
- You cannot "fake" a proof on uncommitted code.
- **Commit first, then Verify.**

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
- **Strict Mode**: Enforced hygiene (no dirty checks)

### v0.4.0 🚧 - The Agent Protocol
- [ ] JSON-RPC interface for Agents
- [ ] Structured "Reasoning" for attestations
- [ ] Remote Sync (Git-backed DB)

---

*"What is true, right now, in this repo?"*