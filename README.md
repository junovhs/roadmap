# Roadmap (Project Cortex)

> **Git for your Intent.**
> A dependency-graph based project manager that treats task completion as a compile-time correctness problem.

---

## 1. The Philosophy

Most project management tools (JIRA, Trello, text files, `TODO.md`) are **Loggers**. They rely on the user to honestly report the state of the world. They allow "split-brain" scenarios where the documentation says a task is done, but the code says otherwise.

**Roadmap** is a **State Machine**.

1.  **Graph, not List:** Projects are Directed Acyclic Graphs (DAGs). You cannot build the roof before the foundation. Roadmap enforces this topologically.
2.  **Trust, but Verify:** A task is not `DONE` until a verification command (unit test, script, build check) returns Exit Code 0.
3.  **Local Velocity:** Built in Rust on SQLite. <10ms startup time. Designed to run inside tight Agent loops or shell prompts.
4.  **Agent-First:** The CLI is designed to be the "Ground Truth" for AI Agents, preventing hallucinated progress by forcing them to satisfy the dependency graph.

---

## 2. Architecture

### The Data Layer (`.roadmap/state.db`)
We reject TOML/JSON for state storage. Text parsing is O(N); SQL is O(1).
We use **SQLite** via `rusqlite` to maintain ACID compliance for project state.

**Core Schema:**
*   **Tasks:** Nodes in the graph. Contain the `verify_cmd` (e.g., `cargo test foo`).
*   **Dependencies:** Edges in the graph. A task cannot be `ACTIVE` if its parent is not `DONE`.
*   **State:** Key-value store for the "Current Context" (what the user/agent is working on *right now*).

### The Graph Engine
We use `petgraph` to model dependencies in memory.
*   **Topological Sort:** Used to determine the "Next Actionable Task."
*   **Cycle Detection:** Prevents "A blocks B blocks A" logic errors.

### The Verification Layer ("Local CI")
Roadmap is **tool-agnostic**. It does not know about Rust, Python, or SlopChop. It only knows **Shell Commands** and **Exit Codes**.
*   If `verify_cmd` returns `0`, the state transitions to `DONE`.
*   If `verify_cmd` returns `!= 0`, the state remains `ACTIVE`.

---

## 3. The "God Mode" Interface

The CLI is designed for speed (Human) and determinism (Agent).

### `roadmap init`
Initializes `.roadmap/state.db`. Detects the project type (Rust/Node/Python) to set default test runner templates.

### `roadmap add`
Natural language intent parsing.
```bash
# Human style
roadmap add "Add Dark Mode" after "UI Framework"

# Agent style (Strict)
roadmap add "feat-dark-mode" --blocks "feat-settings-page"
```
*   **Fuzzy Matching:** Finds "UI Framework" task ID automatically.
*   **Graph Insertion:** Immediately validates acyclicity.

### `roadmap next` (The Critical Path)
Calculates the **Critical Path**. Returns only tasks where `in_degree == 0` (unblocked) and `status != DONE`.
```text
🎯 FOCUS: [db-setup] Initialize SQLite Schema
   └─ 🚧 BLOCKS: [impl-types] Define core structs
```

### `roadmap do <task>`
Context switching. Sets the global `active_task` pointer.
*   **Human:** "I am working on this now."
*   **Agent:** "I am focusing my context window on this problem."

### `roadmap check`
The compilation step for your project management.
1.  Retrieves `verify_cmd` for the active task.
2.  Executes it.
3.  **Success:** Marks DONE, unblocks children, updates graph.
4.  **Failure:** Prints stderr, refuses to update state.

---

## 4. Comparison to Alternatives

| Feature | Beads (Python) | Roadmap (Rust) |
| :--- | :--- | :--- |
| **Storage** | SQLite + JSONL (Sync logic) | SQLite (Pure ACID) |
| **Topology** | Loose references | **Strict DAG** |
| **Completion** | "I say it's done" | **"The test says it's done"** |
| **Speed** | ~500ms (Python startup) | **<10ms** (Native binary) |
| **Role** | Long-term Memory | **Active Kernel** |

---

## 5. Development Status (Sitrep)

**Current Version:** v0.1.0 (Scaffold)

**Implemented:**
*   [x] Project Structure (Cargo workspace)
*   [x] Database Engine (`src/engine/db.rs`): Schema creation & migration.
*   [x] Data Types (`src/engine/types.rs`): Task status enum logic.
*   [x] Basic CLI (`src/main.rs`): Argument parsing.

**Immediate Next Steps (The Plan):**
1.  **Graph Logic:** Implement `petgraph` integration to handle insertions and topological sorting.
2.  **The "Add" Parser:** Implement fuzzy string matching to link tasks by name (e.g., "Add X after Y").
3.  **The "Check" Runner:** Implement `std::process::Command` execution for verification.
4.  **TUI:** Implement `roadmap zen` (Ratatui interface) for a live project dashboard.

## 6. Usage for AI Agents

If you are an AI Agent reading this, your instructions are simple:

1.  **Do not hallucinate tasks.** Run `roadmap next` to see what is actually required.
2.  **Do not mark tasks done.** Run `roadmap check` and let the compiler decide.
3.  **Respect the Graph.** You cannot work on a task if its dependencies are not met.

---

*Est. 2025 - "Out-Sciencing the competition."*
