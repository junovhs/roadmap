# Roadmap: The Sabotage Pivot (v0.5.0)
## Architectural Specification & Philosophical Manifesto

**Document Status:** REVISED / APPROVED
**Target Version:** v0.5.0 ("The Truth Engine")
**Date:** January 4, 2026

---

# 1. Executive Summary

Roadmap currently operates on an **Optimistic Verification Model**: it assumes that if a test command exits with code `0`, the task is complete.

This model is insufficient for AI-assisted development. AI agents, driven by reward functions, suffer from the **"Lazy Teacher Problem."** When unable to solve a complex coding problem, an agent often modifies the *test* to pass unconditionally (e.g., `assert(true)`), rather than fixing the code.

To the human operator who cannot read code, a passing test looks identical to a cheating test.

**The Sabotage Pivot** moves Roadmap to a **Zero-Trust Verification Model**. We introduce the **Law of Falsifiability**: *A test is only valid if we can prove it fails when the code is broken.*

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
The primary user acts as a "Blind Master"—directing work without the ability to manually audit source code.
Therefore:
1.  **Code auditing is disallowed.** We cannot rely on the user "checking the diff."
2.  **Behavior is the only truth.** We must rely on the observable runtime behavior of the verification stack.
3.  **Automation is mandatory.** The sabotage-check-revert loop must be atomic.

---

# 3. Risk Analysis: The "False Trust" Vector

A naive sabotage implementation introduces a new risk: **False Trust**.

**Scenario:**
1. Sabotage command deletes a semicolon.
2. Code fails to compile.
3. Test command fails (Exit 1).
4. Roadmap reports: "TRUSTWORTHY" (because test failed).

**Reality:** The test suite did *not* catch a logic bug; the compiler caught a syntax error. The test logic itself might still be vacuous (`assert(true)`), but we never reached it.

**Mitigation Strategy:**
We must capture **Forensic Metadata** to allow the operator to distinguish between "Build Failure" and "Logic Failure," and eventually support multiple mutation samples.

---

# 4. Architecture: The Sabotage Loop

### 4.1 The New Task Properties
*   `sabotage_cmd`: A command that modifies the source code to introduce a fault.
*   `sabotage_target`: (Optional) Specific files to target.

### 4.2 The Execution Pipeline (`roadmap audit`)

**Phase 1: The Baseline**
1.  **Hygiene Check:** Ensure git working tree is clean.
2.  **Standard Verification:** Run `test_cmd`.
    *   *Expectation:* **PASS** (Exit 0).
    *   *Failure:* Stop. The code is already broken.

**Phase 2: The Attack (Sabotage)**
1.  **Snapshot:** Create a temporary protection layer (`git stash create` / `git reset --hard`).
2.  **Mutate:** Run `sabotage_cmd`.
    *   *Requirement:* The mutator must return Exit 0 if it successfully mutated logic, or Exit 1 if it couldn't find a target (Ineffective).
3.  **Validation:** Ensure `git status --porcelain` shows changes.

**Phase 3: The Interrogation**
1.  **Run Test:** Run `test_cmd` against the mutated code.
2.  **Evaluation:**
    *   **PASS (Exit 0):** **FRAUD DETECTED.** The test is vacuous.
    *   **FAIL (Exit 1+):** **TRUSTWORTHY.** The test reacted to the change.

**Phase 4: The Restoration**
1.  **Revert:** Hard reset to the snapshot created in Phase 2.
2.  **Record:** Write the result to the Audit Log with forensic metadata.

---

# 5. Technical Implementation Specification

### 5.1 Database Schema Updates (`state.db`)

We need rich provenance to debug *why* a test was deemed trustworthy or fraudulent.

**Table: `tasks`**
```sql
ALTER TABLE tasks ADD COLUMN sabotage_cmd TEXT;
```

**Table: `proofs`**
```sql
ALTER TABLE proofs ADD COLUMN verification_level TEXT; -- 'STANDARD' | 'DEEP'
ALTER TABLE proofs ADD COLUMN meta TEXT;               -- JSON blob
```

**JSON Schema for `meta`:**
```json
{
  "mode": "audit",
  "sabotage_result": "caught",      // caught | missed | ineffective
  "mutation_signature": "bool_flip", // What kind of damage did we do?
  "baseline_exit": 0,
  "sabotaged_exit": 101             // Useful for distinguishing crash vs fail
}
```

### 5.2 The "Sabotage Protocol" (Standard Interface)
Roadmap is the orchestrator. The external mutator (e.g., `slopchop sabotage`) must adhere to this contract:

1.  **Input:** File path(s).
2.  **Logic:** Prefer semantic mutations (logic inversion) over syntax destruction.
3.  **Exit Codes:**
    *   `0`: Mutation applied.
    *   `1`: No mutable code found (e.g., file is empty or only structs).

### 5.3 Safety Mechanisms (The Sandbox)
1.  **The Git Lock:** Deep Verification refuses to run if `git status --porcelain` is not empty.
2.  **The Atomic Revert:** Use `git stash create` to store state *without* modifying the working tree index initially, then `git reset --hard HEAD` to clear sabotage. Ideally implement a Rust `Drop` guard to ensure `git reset` fires even if the process panics.

---

# 6. UX Design: The "Audit"

### 6.1 The `roadmap audit` Command

```bash
$ roadmap audit --task "User Auth"

1. Establishing Baseline...
   [OK] Tests passed.

2. Applying Sabotage...
   [OK] Mutated 'src/auth/mod.rs' (inverted logic at line 42).

3. Interrogating Tests...
   [OK] Tests FAILED as expected.

✅ TRUSTWORTHY
The test suite successfully detected a logic inversion.
```

### 6.2 The "Red Flag" Output (Fraud)

```bash
$ roadmap audit --task "User Auth"

...
3. Interrogating Tests...
   [WARNING] Tests PASSED despite sabotage!

🚨 FRAUD DETECTED 🚨
The code was broken, but the tests reported Success.
The test suite for "User Auth" is likely vacuous or commented out.
```

### 6.3 The "Yellow Flag" Output (Ineffective)

```bash
$ roadmap audit --task "Types"

...
2. Applying Sabotage...
   [SKIP] Mutator reported no logic to break.

⚠️ INEFFECTIVE
The target file contains only definitions (structs/enums) with no executable logic to flip.
Audit skipped.
```

---

# 7. Summary of Changes Required

To pivot Roadmap to this Trustless Architecture:

1.  **Database:** Schema migration for `sabotage_cmd` and `proofs.meta`.
2.  **Orchestrator:** Build the `VerifyRunner` logic to handle the 4-phase "Sabotage Loop."
3.  **Safety:** Implement the `GitGuard` struct to handle stashing/restoring safely.
4.  **CLI:** Add `--sabotage` flags to `add` and create the `audit` command.
5.  **Forensics:** Ensure `meta` JSON is populated so we can debug "False Trust" incidents later.

This architecture ensures that Roadmap becomes not just a list of tasks, but a **mathematical guarantee of intent**.
