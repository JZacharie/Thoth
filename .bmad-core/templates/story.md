# Template: User Story

Use this template for all new feature stories in Pylos.

---

## Story: [Title]

**ID**: S[sprint]-[number]
**Epic**: [Epic name]
**Points**: [1 / 2 / 3 / 5 / 8]
**Status**: [Backlog / Ready / In Progress / Review / Done]

---

### User Story

**As a** [Platform Engineer / ML Engineer / Team Lead]
**I want** [specific capability]
**So that** [concrete benefit]

---

### Acceptance Criteria

- [ ] Given [precondition], when [action], then [expected outcome]
- [ ] Given [precondition], when [action], then [expected outcome]
- [ ] Error cases: given [error condition], then [error behavior]

---

### Technical Notes

**Affected crates:**
- [ ] `pylos-core` — [describe changes]
- [ ] `pylos-infrastructure` — [describe changes]
- [ ] `pylos-application` — [describe changes]
- [ ] `pylos-server` — [describe changes]
- [ ] `ui` — [describe changes]

**Config changes** (`pylos.json`):
```json
// Describe new config fields here
```

**New/modified endpoints:**
- `METHOD /path` — description

**New metrics:**
- `metric_name{labels}` — description

**New error variants** (`PylosError`):
- `VariantName` — when it occurs

---

### Implementation Notes

[Any implementation guidance, links to related code, patterns to follow]

---

### Definition of Done

- [ ] Acceptance criteria all met
- [ ] Unit tests written and passing
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all` applied
- [ ] Config schema updated if applicable
- [ ] UI updated if applicable
- [ ] PR reviewed and merged
- [ ] No regressions

---

### Dependencies

- Blocked by: [story IDs or external blockers]
- Blocks: [story IDs]
