# Roadmap (Project Cortex)

> **Git for your Intent.**
> A dependency-graph based project manager that treats task completion as a compile-time correctness problem.

---

## Quick Start

```bash
# Initialize in your project
roadmap init

# Add tasks with dependencies
roadmap add "Setup Database" --test "cargo test db_"
roadmap add "Implement Auth" --after "Setup Database" --test "cargo test auth_"
roadmap add "Build API" --after "Implement Auth" --test "cargo test api_"

# See what's actionable (Critical Path)
roadmap next

# Start working on a task
roadmap do "Setup Database"

# Run verification (the truth oracle)
roadmap check
```

---

## Philosophy

Most project management tools (JIRA, Trello, `TODO.md`) are **Loggers**. They rely on the user to honestly report the state of the world.

**Roadmap** is a **State Machine**.

1. **Graph, not List:** Projects are DAGs. You cannot build the roof before the foundation.
2. **Trust, but Verify:** A task is not `DONE` until `verify_cmd` returns Exit Code 0.
3. **Local Velocity:** Built in Rust on SQLite. <15ms cold start.
4. **Agent-First:** The CLI is "Ground Truth" for AI Agents, preventing hallucinated progress.

---

## Commands

### `roadmap init`
Initialize `.roadmap/state.db` in the current directory.

### `roadmap add <title>`
Add a new task with optional dependencies and verification.

```bash
# Simple task
roadmap add "Write documentation"

# With test command (the oracle)
roadmap add "Fix auth bug" --test "cargo test auth_middleware"

# With dependency (must complete "Auth" first)
roadmap add "Build settings page" --after "Auth"

# This task blocks another (reverse dependency)
roadmap add "Design system" --blocks "Build UI"
```

**Fuzzy Resolution:** Task references can be:
- Exact ID: `42`
- Exact slug: `setup-database`
- Fuzzy match: `"database"`, `"auth"`, `"Setup"`

### `roadmap next`
Show the **Critical Path** — tasks with no incomplete blockers.

```bash
# Human-readable
roadmap next

# Agent-friendly JSON
roadmap next --json
```

### `roadmap do <task>`
Set focus to a task. Validates dependencies are satisfied.

```bash
roadmap do "auth"
# ● Now working on: [implement-auth] Implement Authentication
```

### `roadmap check`
**The Verification Oracle.** Runs `verify_cmd` for the active task.

- **Exit 0:** Task marked `DONE`, children unblocked.
- **Exit ≠ 0:** Task remains `ACTIVE`, stderr displayed.

```bash
roadmap check
# 🔍 Checking: [implement-auth] Implement Authentication
#    running: cargo test auth_
# ✓ Verified! Task [implement-auth] marked DONE (0.42s)
```

### `roadmap list`
Show all tasks with their status.

### `roadmap status`
Overview: completion count, active task, next available.

---

## For AI Agents

If you are an AI Agent reading this:

1. **Do not hallucinate tasks.** Run `roadmap next --json` to see what is actually required.
2. **Do not mark tasks done.** Run `roadmap check` and let the exit code decide.
3. **Respect the Graph.** You cannot work on a task if its dependencies are not met.

### Agent Loop Example

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
SQLite with ACID guarantees. No text parsing — O(1) lookups.

**Schema:**
- `tasks`: Nodes (id, slug, title, status, test_cmd)
- `dependencies`: Edges (blocker_id → blocked_id)
- `state`: Key-value for current context

### Graph Engine (`petgraph`)
- Topological sort for critical path
- Cycle detection at insertion time
- O(V+E) traversal

### Verification Runner
Tool-agnostic. Only understands:
- Shell commands
- Exit codes (0 = success)

---

## Building

```bash
cargo build --release
# Binary at: target/release/roadmap

# Or install globally
cargo install --path .
```

---

## Development Status

**v0.1.0** — Core Implementation

- [x] Database engine (SQLite)
- [x] Graph engine (petgraph, cycle detection)
- [x] Fuzzy task resolution
- [x] Verification runner (shell execution)
- [x] CLI commands: init, add, next, list, do, check, status

**Next:**
- [ ] `roadmap edit` — modify tasks
- [ ] `roadmap zen` — TUI dashboard (Ratatui)
- [ ] `--force` flag for manual overrides

---

*Est. 2025 — "Out-Sciencing the competition."*
