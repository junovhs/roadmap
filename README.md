# Roadmap (Project Cortex)

> **A proof-carrying roadmap.**
> A DAG of claims whose completion is operationally trustworthy.
>
> Said simply: it's a roadmap for your project that **updates itself based on what is and isn't true.**

---

## The Workflow: From Zero to Truth

Roadmap isn't a todo list; it's a state machine. Here is the exact lifecycle of a project, from empty directory to verified truth.

### 1. Define the Reality (Setup)
You define the work and the topology (dependencies). This replaces writing a `TODO.md` or filling out JIRA tickets.

```bash
roadmap init

# Define the goal, the test that proves it, and the files it touches.
roadmap add "Database Engine" \
    --test "cargo test db_core" \
    --scope "src/db/**"

# Define the dependent feature. This is BLOCKED until DB is done.
roadmap add "User Auth" \
    --after "database-engine" \
    --test "cargo test auth" \
    --scope "src/auth/**"
```

### 2. The Daily Loop (Flow)
You don't guess what to work on. You ask the graph.

```bash
roadmap next
# Output: [database-engine] (User Auth is hidden because it's blocked)

roadmap do database-engine
# Locks this task as your active context.
```

### 3. The Handshake (Hygiene)
You write the code. Tests pass.
**Crucial Step:** Roadmap adheres to the **Law of Hygiene**. It refuses to verify a "dirty" working tree. Truth is a property of a *Commit*, not a work-in-progress.

```bash
git add .
git commit -m "feat: db works"

roadmap check
# Runs 'cargo test db_core'.
# If Pass: Records {SHA, Timestamp, ExitCode} to SQLite.
# 'User Auth' is now unblocked.
```

### 4. The Payoff (Truth Decay)
Three weeks later, you change a file in `src/db/lib.rs`.
Roadmap detects that `HEAD` moved and the diff touches the scope of "Database Engine".

```bash
roadmap status
# Output: [database-engine] -> STALE (yellow)
```

The system forces you to re-verify the foundation before you build more on top. This prevents the "Rot" that plagues standard todo lists.

---

## Philosophy

### The Core Difference

| Tool | Model | "Done" means... |
|------|-------|-----------------|
| JIRA, Trello, TODO.md | Logger | "Someone said so" |
| **Roadmap** | State Machine | "The verifier passed on this Commit" |

### 1. Derived Truth
Roadmap trusts the **Proof Audit Log**, not your memory.
- **PROVEN:** The verification passed on the *current* Git commit.
- **STALE:** The verification passed previously, but the code has changed since then.
- **BROKEN:** The verification command failed.

### 2. Smart Decay (Contextual Intelligence)
Codebases change. Roadmap knows *what* changed.
- **Global Decay:** If a task has no scope, *any* commit marks it STALE. (Safe default).
- **Smart Decay:** If a task has a `--scope` (e.g., `src/auth/**`), it stays PROVEN unless the changes touch those files.

---

## Commands

| Command | Description |
|---------|-------------|
| `roadmap add` | Add claim with `--after`, `--test`, `--scope` |
| `roadmap next` | Show frontier (unblocked, unproven) |
| `roadmap do` | Set active claim (validates deps) |
| `roadmap check` | Run `prove_cmd`, store proof, update status |
| `roadmap why` | Explain why a task is Stale/Proven + Audit Log |
| `roadmap stale` | **Debt Radar:** Scan for invalidated proofs |
| `roadmap history` | Stream chronological verification events |
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

---

*"What is true, right now, in this repo?"*
