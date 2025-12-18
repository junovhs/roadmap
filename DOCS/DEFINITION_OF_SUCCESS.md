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
3. The proof is still valid for the current `HEAD` (Smart Decay)

---

## 2. Version Milestones

### v0.1.0 ✅ - MVP
Scaffold, CLI, DAG enforcement.

### v0.1.1 ✅ - Ship-Worthy
Transactions, WAL mode, Fuzzy resolution.

### v0.2.0 ✅ - Derived Truth
- [x] **Audit Log**: Append-only proof history (`proofs` table)
- [x] **Truth Decay**: SHA-based staleness checks
- [x] **Visibility**: `why`, `stale`, and `history` commands

### v0.3.0 ✅ - Contextual Intelligence
- [x] **Smart Decay**: Scoped invalidation using `git diff`
- [x] **RepoContext**: Efficient git operations
- [x] **Strict Mode**: `check` fails on dirty repo

### v0.4.0 - "The Agent Protocol"
Making Roadmap the standard interface for AI coding agents.

- [ ] **Structured Output**: Full JSON schema for all read commands
- [ ] **Agent Handshake**: Protocol for agents to "sign" their work
- [ ] **Remote Sync**: Database replication via Git LFS or similar

---

## 3. Technical Constraints

### Performance
- Cold start: < 15ms
- Graph traversal (1000 nodes): < 5ms
- Memory: < 10MB RSS

### Integrity
- SQLite strict transactions
- Cycle rejection at insertion time
- **Law of Hygiene**: No proofs on dirty repos

### SlopChop Compliance
- Max file tokens: 2000
- Max cyclomatic complexity: 8
- Max nesting depth: 3

---

## 4. Sign-off Criteria

### v0.3.0 ships when:
1. `roadmap check` fails on uncommitted changes.
2. Changing a scoped file marks the task STALE.
3. Changing an unscoped file does NOT mark scoped tasks STALE.

---

*"What is true, right now, in this repo?"*