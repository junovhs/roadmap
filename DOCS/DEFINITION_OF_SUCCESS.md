# Definition of Success: Project Cortex (Roadmap V3)

> **Objective:** Obsolesce "Logger" based project management (Beads, TODO.md) by establishing a "State Machine" based project kernel.

## 1. Executive Summary
Success is defined by shifting the burden of truth from **User Testimony** ("I say it's done") to **Computational Verification** ("The exit code is 0"). Roadmap V3 succeeds if it acts as a non-bypassable gatekeeper for project progress, serving both Human and Artificial Intelligence agents with zero latency.

---

## 2. Technical Metrics (The Hard Numbers)

To "out-science" the competition, we rely on Rust's zero-cost abstractions and SQLite's speed.

### 2.1 Latency & Performance
*   **Cold Start:** The CLI must initialize and execute a query (`roadmap next`) in **< 15ms**.
    *   *Context:* Beads (Python) incurs interpreter startup costs (~200ms+). Roadmap must feel instantaneous, suitable for integration into shell prompts (`PS1`) or git hooks.
*   **Graph Traversal:** Topological sort of a graph with 1,000 nodes and 2,000 edges must complete in **< 5ms**.
*   **Memory Footprint:** The resident set size (RSS) during idle/query operations must remain **< 10MB**.

### 2.2 Data Integrity (ACID vs Eventual)
*   **Consistency:** Zero tolerance for "Split Brain."
    *   *Success:* Unlike Beads (which uses append-only logs for distributed merge conflict resolution), Roadmap uses SQLite strict transactions. The state on disk is always valid.
*   **Cycle Prevention:** The system **must reject** any `roadmap add` command that introduces a cycle. This check must happen at insertion time, not read time.

### 2.3 Structural Integrity
*   **SlopChop Compliance:** The Roadmap codebase itself must adhere to the 3 Laws.
    *   Max Cyclomatic Complexity: **8**
    *   Max File Tokens: **2000**
    *   Max Nesting Depth: **3**

---

## 3. The "Killer Feature" Metrics

### 3.1 The "Next" Heuristic (Critical Path Analysis)
Success is when an Agent *never* has to guess what to do.
*   **Constraint:** `roadmap next` must return **strictly** nodes where `in_degree == 0` (excluding DONE parents).
*   **Metric:** An Agent following `roadmap next` instructions blindly must face **0% Blockage Errors**.

### 3.2 Verification Gatekeeping
*   **The Pivot:** A task is not done until `verify_cmd` returns `0`.
*   **Metric:** In Strict Mode, `roadmap check` is the **only** mechanism to transition a task from `ACTIVE` to `DONE`.

### 3.3 Fuzzy Resolution (The UX Bridge)
Humans and Agents are imprecise. The tool must be forgiving on input, strict on output.
*   **Input:** `roadmap add "Auth" after "Database"`
*   **Resolution:** The system must resolve "Database" to task ID via:
    1.  Exact ID Match
    2.  Exact Slug Match
    3.  Fuzzy Match (substring, word overlap, character similarity)
*   **Ambiguity:** If resolution confidence is < 40%, the tool must error with suggestions.

---

## 4. Agent Interaction Protocol

### 4.1 The Context Window Victory
*   **Problem:** Agents reading `TODO.md` waste tokens reading completed/blocked tasks.
*   **Solution:** `roadmap next --json`
*   **Metric:** Context payload reduced by **> 90%** — only the *frontier* of the graph.

### 4.2 Hallucination Containment
*   **Scenario:** Agent claims "I fixed the bug."
*   **Response:** System runs `roadmap check`. Test fails. System rejects the transition.
*   **Success:** Prevents the "Lying Agent" phenomenon by grounding status in execution.

---

## 5. Sign-off Criteria

We are done when:
1.  We can run `roadmap init` in a new repo.
2.  We can script a graph of 5 dependent tasks using fuzzy names.
3.  We can fail a test, run `roadmap check`, and see the status remain `ACTIVE`.
4.  We can pass a test, run `roadmap check`, and see the status flip to `DONE`.
