# DDD & Clean Architecture

An [Agent Skill](https://agentskills.io) that makes coding agents such as Claude Code build and review backend services with **Domain-Driven Design**, **hexagonal architecture**, **CQRS** and **domain events**. All examples are in Rust.

## What it does

- Lays the code out as bounded contexts (one crate each) and modules (one aggregate each), with `domain`, `application` and `infrastructure` layers.
- Models aggregates with validated Value Objects, and handles optional concepts inside the VO.
- Declares ports as traits in the domain and keeps adapters in infrastructure, so dependencies always point inward.
- Runs every use case as a command or a query through a bus, and announces state changes with domain events published after persisting.
- Wires dependencies through constructors in one composition root per app.
- Tests use cases with hand-written mocks of the ports and Object Mothers, and checks every adapter against the same contract.
- Reviews existing code against these rules, with a checklist and an anti-pattern table per topic.

## What's inside

| File | Topic |
|---|---|
| [`SKILL.md`](SKILL.md) | Core rules, workflow, naming conventions and review guide |
| [`references/project-structure.md`](references/project-structure.md) | Bounded contexts, modules, shared modules and crates, apps, dependency rules |
| [`references/hexagonal-architecture.md`](references/hexagonal-architecture.md) | Abstractions, interfaces and traits, dependency inversion, layers, use cases, domain services |
| [`references/value-objects.md`](references/value-objects.md) | Value Objects, aggregate roots, nullable concepts |
| [`references/cqrs.md`](references/cqrs.md) | Commands, queries, handlers, buses, entry points |
| [`references/dependency-injection.md`](references/dependency-injection.md) | Constructor injection, injected buses, the composition root |
| [`references/domain-events.md`](references/domain-events.md) | Recording, publishing and subscribing to domain events |
| [`references/testing.md`](references/testing.md) | Mocks, Object Mothers, unit, contract and end-to-end tests |

## Installation

Clone the repository into your skills directory as `ddd-clean-architecture`:

```bash
git clone https://github.com/otsobide/ddd-clean-architecture-skill.git ~/.claude/skills/ddd-clean-architecture
```

To install it for a single project, clone it into `.claude/skills/ddd-clean-architecture` inside that project instead.
