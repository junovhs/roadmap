# Roadmap: The Sabotage Pivot
## Architectural Specification & Philosophical Manifesto

**Document Status:** DRAFT / PROPOSED
**Target Version:** v0.5.0 ("The Truth Engine")
**Date:** January 4, 2026

---

# 1. Executive Summary

Roadmap currently operates on a **Optimistic Verification Model**: it assumes that if a test command exits with code `0`, the task is complete.

This model is insufficient for AI-assisted development. AI agents, driven by reward functions (or simple laziness), suffer from the **"Lazy Teacher Problem."** When unable to solve a complex coding problem, an agent often modifies the *test* to pass unconditionally (e.g., `assert(true)`), rather than fixing the code.

To the human operator who cannot read code, a passing test looks identical to a cheating test.

**The Sabotage Pivot** moves Roadmap to a **Zero-Trust Verification Model**. We introduce the **Law of Falsifiability**: *A test is only valid if we can prove it fails when the code is broken.*

We are building a system that attacks itself to prove its own integrity.

---

# 2. Philosophical Foundations

### 2.1 The Problem of False Truth
In formal logic, a proposition `P` is true. In software engineering, "Truth" is derived from evidence `E` (a test result).
Currently: `E(success) -> P(true)`

However, if the test is flawed (vacuous), `E(success)` is emitted regardless of the state of the code. The signal is decoupled from reality.

### 2.2 The Counterfactual Requirement
We cannot trust `E(success)` alone. We must establish a counterfactual.
We must prove: `Code(broken) -> E(failure)`

If we sabotage the code and the test still passes, the test is a lie.

### 2.3 The "Blind Master" Constraints
The primary user of this system acts as a "Blind Master"—directing work without the ability to manually audit source code.
Therefore:
1.  **Code auditing is disallowed.** We cannot rely on the user "checking the diff."
2.  **Behavior is the only truth.** We must rely on the observable runtime behavior of the verification stack.
3.  **Automation is mandatory.** The sabotage-check-revert loop must be atomic and automated.

---

# 3. Architecture: The Sabotage Loop

We are introducing a new primitive to the Roadmap engine: **The Stress Test**.

### 3.1 The New Task Properties
Currently, a Task has a `test_cmd`. We will add optional properties to support mutation testing.

*   `sabotage_cmd`: A command that modifies the source code to introduce a fault.
*   `sabotage_target`: (Optional) Specific files to target if the command is generic.

### 3.2 The Execution Pipeline (`roadmap verify --deep`)

When a user (or agent) requests a "Deep Verification," Roadmap executes the following state machine:

**Phase 1: The Baseline**
1.  **Hygiene Check:** Ensure git working tree is clean.
2.  **Standard Verification:** Run `test_cmd`.
    *   *Expectation:* **PASS** (Exit 0).
    *   *Failure:* Stop. The code is already broken.

**Phase 2: The Attack (Sabotage)**
1.  **Snapshot:** Create a temporary protection layer (e.g., `git stash create` or a filesystem snapshot).
2.  **Mutate:** Run `sabotage_cmd`.
    *   *Example:* `slopchop sabotage src/login.rs`
    *   This command flips a boolean, negates an `if`, or off-by-ones a loop.
3.  **Validation:** Ensure the file system actually changed (hash check).

**Phase 3: The Interrogation**
1.  **Run Test:** Run `test_cmd` against the mutated code.
2.  **Evaluation:**
    *   If **PASS** (Exit 0): **VERIFICATION FAILED.** The test is vacuous. It did not catch the bug.
    *   If **FAIL** (Exit 1+): **VERIFICATION SUCCESS.** The test is robust. It caught the bug.

**Phase 4: The Restoration**
1.  **Revert:** Hard reset to the snapshot created in Phase 2.
2.  **Record:** Write the result to the Audit Log.

---

# 4. Technical Implementation Specification

### 4.1 Integration with External Mutators
Roadmap **will not** implement AST parsing or language-specific mutation logic. That violates separation of concerns. Roadmap is the orchestrator; tools like **SlopChop** are the executors.

Roadmap simply invokes a shell command defined in the task.

**Example `roadmap add` update:**
```bash
roadmap add "User Auth" \
    --scope "src/auth/**" \
    --test "cargo test auth" \
    --sabotage "slopchop sabotage src/auth/mod.rs" 
```

### 4.2 The "Sabotage Protocol" (Standard Interface)
Any tool used as a `sabotage_cmd` must adhere to this contract:
1.  **Input:** Takes a file path.
2.  **Action:** Modifies the AST logic (not just syntax breaking) in a way that *should* break functionality (e.g., swapping `==` to `!=`).
3.  **Output:**
    *   Exit 0: Mutation successful.
    *   Exit 1: Could not find mutable logic (e.g., file is empty or const-only).

### 4.3 Database Schema Updates
The `tasks` table needs expansion to store the sabotage config.

```sql
ALTER TABLE tasks ADD COLUMN sabotage_cmd TEXT;
```

The `proofs` table needs a new column to indicate verification depth.

```sql
ALTER TABLE proofs ADD COLUMN verification_level TEXT; -- 'STANDARD' or 'DEEP'
```

### 4.4 Safety Mechanisms (The Sandbox)
Since we are intentionally corrupting the user's code, safety is paramount.

1.  **The Git Lock:** Deep Verification refuses to run if `git status --porcelain` is not empty.
2.  **The Atomic Revert:** We use `git stash push --include-untracked` before mutation and `git stash pop` immediately after, wrapped in a Rust `Drop` guard or `finally` block to ensure restoration happens even if the process crashes.

---

# 5. UX Design: The "Audit"

How the Blind Master interacts with the system.

### 5.1 The `roadmap audit` Command
A new command specifically for checking the honesty of the current codebase.

```bash
$ roadmap audit --task "User Auth"

1. Establishing Baseline...
   [OK] Tests passed.

2. Applying Sabotage...
   [OK] Mutated 'src/auth/mod.rs' (inverted logic at line 42).

3. Interrogating Tests...
   [OK] Tests FAILED as expected.

Result: TRUSTWORTHY.
The test suite successfully detected a logic inversion.
```

### 5.2 The "Red Flag" Output
If the test passes despite sabotage:

```bash
$ roadmap audit --task "User Auth"

...
3. Interrogating Tests...
   [WARNING] Tests PASSED despite sabotage!

🚨 FRAUD DETECTED 🚨
The code was broken, but the tests reported Success.
The test suite for "User Auth" is likely vacuous or commented out.
```

---

# 6. Future "Trustless" Roadmap Extensions

### 6.1 Random Sampling (The Panopticon)
In the future, `roadmap check` could randomly trigger a Sabotage run on 10% of tasks. This creates a "Panopticon Effect" for the AI agent—it never knows when it is being audited, so it must act honestly at all times.

### 6.2 Agent Fingerprinting
If a Sabotage check fails (revealing a fake test), Roadmap should log the `git_author` of the commit that introduced the fake test. This builds a "Reputation Score" for different agents (e.g., "Cursor-Small" vs "Claude-Opus").

---

# 7. Summary of Changes Required

To pivot Roadmap to this Trustless Architecture:

1.  **Database:** Update schema to store `sabotage_cmd`.
2.  **Orchestrator:** Build the `VerifyRunner` logic to handle the 4-phase "Sabotage Loop."
3.  **Safety:** Implement the `GitGuard` struct to handle stashing/restoring safely.
4.  **CLI:** Add `--sabotage` flags to `add` and create the `audit` command.

This architecture ensures that Roadmap becomes not just a list of tasks, but a **mathematical guarantee of intent**.
