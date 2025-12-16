# Definition of Success: Project Cortex (Roadmap)

> **Thesis:** Roadmap is a proof-carrying roadmap - a DAG of claims whose completion is operationally trustworthy.

---

## 1. Core Philosophy

### The Differentiator

| Tool | Question it Answers |
|------|---------------------|
| Beads | "What should we do next, and how do we remember it?" |
| **Roadmap** | "What is true right now, and what truth is missing?" |

### DONE as Derived Fact

"DONE" is not a flag someone sets. It is a **computed state** based on:

1. A verifier exists (`prove_cmd`)
2. The verifier passed (exit code 0)
3. The proof is still valid (repo state hasn't invalidated it)

### The Claim Model

Everything in Roadmap is a **Claim** - a statement about the project that can be proven.

```
Claim {
    statement: "POST /login rejects invalid credentials"
    prove_cmd: "cargo test auth::test_login_rejection"
    scope: ["src/auth/**"]  // what changes invalidate this
    depends_on: [other_claim_ids]
}
```

**Derived States:**
- `UNPROVEN` - no proof exists
- `PROVEN` - proof passed, still valid for current HEAD
- `STALE` - proof passed, but scoped files changed since
- `BROKEN` - proof ran and failed

---

## 2. Version Milestones

### v0.1.0 ? (Current)
MVP scaffold. CLI works, DAG enforced, verification gates completion.

### v0.1.1 - "Ship-Worthy"
Earn the "trustworthy" promise with minimal additions:

- [ ] **Proof evidence capture**: Store `{cmd, exit_code, sha, timestamp}` on check
- [ ] **DB hardening**: `foreign_keys=ON`, WAL mode, busy_timeout
- [ ] **Transactions**: Wrap add + deps + cycle check atomically
- [ ] **Fuzzy strict mode**: `--json` returns error + candidates, never guesses
- [ ] **Rename**: `get_critical_path()`  `get_frontier()`

### v0.2.0 - "Derived Truth"
Status becomes computed, not stored:

- [ ] **Computed status**: UNPROVEN/PROVEN/STALE/BROKEN from proof + HEAD
- [ ] **Scope field**: Define what files invalidate a proof
- [ ] **`roadmap stale`**: Scan for invalidated proofs
- [ ] **Rename internally**: Task  Claim, test_cmd  prove_cmd

### v0.3.0 - "Attestation & Audit"
Handle manual overrides without destroying trust:

- [ ] **ATTESTED state**: `--force` creates ATTESTED (not PROVEN) with reason
- [ ] **Audit log**: Append-only proof history
- [ ] **`roadmap why <claim>`**: Show proof chain

---

## 3. Technical Constraints

### Performance
- Cold start: < 15ms
- Graph traversal (1000 nodes): < 5ms
- Memory: < 10MB RSS

### Integrity
- SQLite strict transactions
- Cycle rejection at insertion time
- No `unwrap()` - all errors handled

### SlopChop Compliance
- Max file tokens: 2000
- Max cyclomatic complexity: 8
- Max nesting depth: 3

---

## 4. Sign-off Criteria

### v0.1.1 ships when:
1. `roadmap check` persists proof evidence
2. DB uses transactions + WAL
3. Fuzzy resolver hard-fails on ambiguity in `--json` mode

### v0.2.0 ships when:
1. `roadmap next` only shows UNPROVEN/STALE claims
2. Changing a file in scope auto-marks claims STALE
3. `DONE` no longer exists as stored state

---

*"What is true, right now, in this repo?"*