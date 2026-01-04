# Roadmap: The Truth Engine

## Standalone Architectural + Philosophical Specification

**Status:** Future Spec / North Star
**Core premise:** Roadmap is a **proof-carrying todo list**. “Done” is derived from reproducible evidence, not user assertion.

---

## 1. Mission

Roadmap exists to prevent three failure modes that appear when code is produced by untrusted agents (including AI):

1. **False completion:** “Done” is asserted without evidence.
2. **Silent regression:** features can be lost/broken without being noticed.
3. **Evidence fraud:** tests or checks are altered to pass regardless of reality.

Roadmap must let a “Blind Master” operator drive complex work without reading code.

---

## 2. First Principles

### 2.1 Evidence over assertion

A task is not complete because someone says it is. A task is complete only when its proof succeeds.

### 2.2 Behavior is the only truth

Roadmap does not rely on diffs, code review, or human auditing. It relies only on observable runtime behavior (the outputs of verification processes).

### 2.3 The Law of Falsifiability

A proof is not trustworthy unless it fails when the system is broken in a relevant way.

### 2.4 Rule of Least Power

Roadmap should prefer the weakest mechanism that reliably enforces truth:

* simple exit codes and minimal structured output,
* minimal state,
* minimal coupling to languages/toolchains.

### 2.5 Antifragile bottom-up evolution

Roadmap must support loose intent that hardens into invariants over time. It must not force premature specification.

---

## 3. Core Abstraction

### 3.1 Task = Claim + Falsifier

A Roadmap task is a **claim** about the system plus a **falsifier** (a command/procedure that would fail if the claim becomes false).

* **Claim:** human-readable, externally meaningful statement.
* **Falsifier:** executable verification method.
* **Scope:** what changes are likely to invalidate the claim (paths/globs/tags).
* **Audit method:** how to test whether the falsifier is honest (optional, but required for “trustworthy”).

This yields a todo list that “completes itself”:

* when the falsifier passes, the task is “done,”
* if it later fails, the task becomes “not done” again.

---

## 4. Task Maturity Model

Roadmap must support progressive formalization with explicit states:

1. **INTENT**

   * “I want this to be true.”
   * No falsifier required yet.
   * Used to capture direction without tests hell.

2. **SPECIFIED**

   * Claim is written in falsifiable terms and a falsifier command exists.
   * It may be failing. That’s acceptable.

3. **PROVEN**

   * Falsifier passes. Task is “done” under standard verification.

4. **AUDITED**

   * Falsifier passes and has passed a falsifiability audit (sabotage/mutation).
   * Only this state may be labeled **TRUSTWORTHY** (see §7).

This model prevents self-defeat: you can be loose early (INTENT), and strict only when you’re making commitments.

---

## 5. Verification Interfaces

### 5.1 Standard verification

Standard verification answers only: “Does the falsifier pass right now?”

* Run the task’s `test_cmd` (or equivalent).
* Record outcome and metadata.
* Update task state to PROVEN or DISPROVEN.

### 5.2 Audit verification (deep)

Audit verification answers: “Is the falsifier lying?”

This is the sabotage/mutation loop (see §6). Importantly, Roadmap does **not** implement language-specific mutation logic; it orchestrates external mutators or scripts via a simple protocol.

---

## 6. The Sabotage Loop (Standalone)

Roadmap implements a 4-phase state machine for `roadmap audit`:

### Phase 1: Baseline

* Verify the environment is eligible (see §6.5).
* Run `test_cmd`. Must PASS, otherwise stop (“already broken”).

### Phase 2: Attack

* Create a reversible snapshot (see §6.5).
* Run `sabotage_cmd` (mutator).
* Confirm that a meaningful change occurred (diff/hash check).

### Phase 3: Interrogation

* Run `test_cmd` on sabotaged state.
* Classify result:

  * PASS ⇒ **FRAUD** (falsifier did not react)
  * FAIL ⇒ potentially **CAUGHT** (subject to failure classification)

### Phase 4: Restoration

* Restore the snapshot atomically.
* Record forensic metadata.

### 6.1 Sabotage Protocol (mutator contract)

A valid `sabotage_cmd` must satisfy:

* **Input:** file path(s) or a task-defined scope.
* **Effect:** attempts a semantic mutation (logic inversion / predicate swap / boundary change), not mere syntax break.
* **Exit codes:**

  * `0`: mutation applied
  * `1`: ineffective (no mutable logic found)
  * `2+`: error (tool failure)

### 6.2 Evidence Adapter (minimal structured output)

Exit codes alone are not enough to achieve “trustworthy,” because “FAIL” can mean “build broke” or “runner crashed” rather than “tests detected semantic failure.”

To keep KISS while enabling strict semantics, Roadmap defines an optional but recommended adapter:

* `test_cmd` may emit a single-line JSON summary to stdout (or write it to a file path provided by Roadmap), e.g.:

  * `{"tests_executed": 42, "failures": 1, "errors": 0, "duration_ms": 12345}`

If no structured evidence is provided, Roadmap must downgrade many outcomes to **INCONCLUSIVE** rather than pretending.

This is the “least power” way to avoid fragile log parsing while still allowing rigorous classification.

### 6.3 Scope validation

For audit integrity, Roadmap should ensure the sabotage target is within the declared task scope (or explicitly opt out). This prevents “audit theater” that mutates irrelevant code.

### 6.4 Determinism posture

Roadmap does not require perfect determinism, but it must record enough metadata (seed, durations, exit codes, summaries) to detect flakiness and to avoid granting “trustworthy” when results are inconsistent.

### 6.5 Safety without SlopChop: snapshotting options

Roadmap must not corrupt the operator’s working environment. It provides safety via snapshot/restore. Options, in order:

1. **Git snapshot (preferred)**

   * Preconditions: repository is a Git repo and working tree is clean.
   * Snapshot: create a temporary stash / commit / or detached worktree.
   * Restore: hard reset / drop worktree / pop stash.
   * Roadmap refuses audit if the tree is dirty, unless an explicit “unsafe” flag is used.

2. **Filesystem snapshot (fallback)**

   * Preconditions: configured workspace root and allowed paths.
   * Snapshot: copy relevant scope to a temp dir.
   * Restore: overwrite from snapshot.
   * More expensive, but dependency-free.

If neither safety method is available, Roadmap must refuse deep audit. (Trustlessness requires safety; otherwise the tool becomes dangerous.)

---

## 7. Verdict Semantics

Roadmap must separate *passing* from *trustworthy*.

### 7.1 Standard outcomes

* **PROVEN:** standard verification passes.
* **DISPROVEN:** standard verification fails.

### 7.2 Audit outcomes

Audit produces exactly one of:

* **TRUSTWORTHY**
  Allowed only when Roadmap can prove all of:

  1. baseline passed
  2. mutation applied (confirmed diff)
  3. build/runner progressed to “tests executed” state (via adapter evidence or reliable classification)
  4. at least one test failure occurred under sabotage
  5. restoration succeeded

* **FRAUD**
  Mutation applied and sabotaged run still passes (no failures).

* **INEFFECTIVE**
  Sabotage tool reports it could not find mutable logic (exit 1). This is not blame; it is information.

* **INCONCLUSIVE**
  Anything else: build failure, runner error, crash, timeout, missing evidence, inconsistent results, inability to confirm tests executed, restoration uncertainty.

### 7.3 “Mathematically impossible to be a lie” (the precise meaning)

Roadmap may only display “TRUSTWORTHY” if it is asserting a statement that is logically entailed by recorded evidence and by the declared trust assumptions:

* The command executor is faithful (Roadmap is actually running what it says it runs).
* The snapshot/restore mechanism is sound.
* The evidence adapter output is not forged (or the executor environment is trusted).

Roadmap should print these assumptions in its documentation once, globally, not spam them per run.

---

## 8. Data Model and Forensics

Roadmap needs an append-only audit trail because “trust” must be inspectable without reading code.

### 8.1 Tasks table

Minimum columns:

* `id`, `title`, `claim`, `scope`
* `test_cmd`
* `sabotage_cmd` (nullable)
* `maturity_state` (`INTENT|SPECIFIED|PROVEN|AUDITED`)
* `created_at`, `updated_at`

### 8.2 Proofs table (append-only)

Minimum columns:

* `id`, `task_id`, `timestamp`
* `verification_level` (`STANDARD|AUDIT`)
* `result` (`PROVEN|DISPROVEN|TRUSTWORTHY|FRAUD|INEFFECTIVE|INCONCLUSIVE`)
* `meta` (JSON blob)

### 8.3 `meta` content (forensic minimum)

* `baseline_exit`, `sabotaged_exit`
* `mutation_applied` (bool), `mutated_files` (list), `mutation_signature` (string if provided)
* `tests_executed`, `failures`, `errors` (if adapter present)
* `failure_class` (`test_fail|build_fail|runner_fail|crash|timeout|unknown`)
* `duration_ms` baseline/sabotaged
* `snapshot_method` (`git|fs`), `restore_ok` (bool)

This allows you to distinguish “compiler caught it” vs “tests caught it” without reading code.

---

## 9. CLI Surface (Minimum Viable)

Roadmap should remain small. A minimal but complete CLI:

* `roadmap add`
  Creates INTENT or SPECIFIED tasks (depending on whether `test_cmd` is provided).

* `roadmap prove` (or `roadmap check`)
  Runs standard verification for a task or set of tasks.

* `roadmap audit`
  Runs the sabotage loop for a task (requires `sabotage_cmd` + safe snapshot mode).

* `roadmap status`
  Shows task states derived from latest proofs.

* `roadmap log`
  Displays forensic proof history for a task.

Optional:

* `roadmap promote` (INTENT→SPECIFIED, SPECIFIED→PROVEN when evidence exists)
* `roadmap sample-audit` (random audits for antifragile “panopticon” behavior)

---

## 10. How You Should Work (Operator Workflow)

This is the practical workflow designed for “AI writes code; I govern truth.”

### 10.1 You write intentions; AI proposes implementations

* Capture goals in INTENT tasks (cheap, loose).
* When a goal becomes important, require the AI to attach:

  * a falsifier (`test_cmd`)
  * and ideally a falsifier-evidence adapter (JSON summary)

### 10.2 You promote only what you cannot afford to lose

Not everything deserves a Roadmap invariant. Only promote:

* user-visible behavior,
* safety/security invariants,
* core product identity.

Everything else can remain narrative in docs or INTENT in Roadmap.

### 10.3 You demand audits only for critical claims

* “PROVEN” is enough for exploratory work.
* “TRUSTWORTHY” is for commitments.

### 10.4 You do not read code

You decide:

* what must be true,
* what scope it applies to,
* and what evidence is acceptable.

Roadmap enforces the rest.

---

## 11. Compatibility with Rust `///` (Optional, Not Required)

If the codebase is Rust, AI can encode claims as `///` docs with doctests. Roadmap does not need to understand Rust; it only needs to run falsifiers (`cargo test`, subsets, or adapter-wrapped commands). Doctests simply become one class of falsifier.

The key principle remains: **Roadmap trusts behavior, not prose.**

---

## 12. Non-Goals

Roadmap explicitly does **not** attempt to:

* infer correctness from code structure,
* implement mutation testing itself (it orchestrates),
* replace a full project management tool,
* predict all edge cases up front.

---

## 13. The North Star Guarantee

When Roadmap marks a task **TRUSTWORTHY**, it is asserting:

* the task’s claim was satisfied by evidence,
* the evidence mechanism was audited for falsifiability,
* and the audit outcome is not ambiguous under Roadmap’s strict classification rules.

Everything else is either “not yet proven” or “proven but not audited.”

That is how you get a todo list that completes itself without becoming a religion.
