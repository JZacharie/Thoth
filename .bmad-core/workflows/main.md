# BMAD Workflow for Pylos

## Overview

This document describes the BMAD (Build More, Architect Dreams) development workflow
adapted for the Pylos AI Gateway project.

## Agents

| Agent | File | Responsibilities |
|---|---|---|
| Architect (Larry) | `.bmad-core/agents/architect.md` | System design, ADRs, trait interfaces |
| Developer (Dev) | `.bmad-core/agents/developer.md` | Implementation, tests, PRs |
| Product Manager (PM) | `.bmad-core/agents/pm.md` | Stories, acceptance criteria, backlog |
| Scrum Master (SM) | `.bmad-core/agents/scrum-master.md` | Sprint planning, DoD, blockers |

## Templates

| Template | File | Use When |
|---|---|---|
| User Story | `.bmad-core/templates/story.md` | Defining any new feature or bug fix |
| New Provider | `.bmad-core/templates/new-provider.md` | Adding an LLM provider |
| ADR | `.bmad-core/templates/adr.md` | Making a significant arch decision |

## Development Workflow

```
1. IDEATE       → PM writes a user story using story.md template
                  Stored in: .bmad-core/data/stories/

2. DESIGN       → Architect reviews story, writes technical notes,
                  creates ADR if needed
                  ADRs stored in: .bmad-core/data/adrs/

3. READY        → SM confirms Definition of Ready is met
                  Story moves to sprint backlog

4. IMPLEMENT    → Developer implements following developer.md conventions
                  Branch: feature/S[N]-[story-id]-[short-title]

5. REVIEW       → PR opened, CI must pass:
                    cargo fmt --all
                    cargo clippy --workspace -- -D warnings
                    cargo test --workspace

6. DONE         → Merged, SM marks story Done
```

## Data Directory Structure

```
.bmad-core/data/
├── stories/           # User stories (story.md instances)
│   └── S1-01-*.md
├── sprints/           # Sprint plans
│   └── sprint-1.md
├── adrs/              # Architecture Decision Records
│   └── adr-001-*.md
└── epics/             # Epic definitions
    └── epic-*.md
```

## Pylos-Specific BMAD Rules

### For any new LLM provider:
1. Use `.bmad-core/templates/new-provider.md` as checklist
2. Architect must approve the domain model changes first
3. Provider must pass all existing integration tests

### For any config schema change:
1. Must be backward-compatible (use serde defaults)
2. Document new fields in `pylos.json` with comments
3. Hot-reload must work without restart

### For any new HTTP endpoint:
1. Add to the endpoints table in `.bmad-core/bmad-project.md`
2. Must include Prometheus metrics
3. Must have tracing spans

### For UI changes:
1. Must work with the existing React/TanStack Query patterns
2. No new direct `fetch` calls — use axios via `ui/src/api/`
3. Responsive layout required (TailwindCSS)

## Quick Start Commands

```bash
# Build
cargo build --release

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Local stack (gateway + UI + monitoring)
docker compose up

# Hot-reload config
curl -X POST http://localhost:3000/config/reload

# View logs
curl http://localhost:3000/api/logs | jq

# View metrics
curl http://localhost:3000/metrics
```

## Resources

- Project context: `.bmad-core/bmad-project.md`
- BMAD docs: https://bmadcode.com/
- Pylos README: `README.md`
