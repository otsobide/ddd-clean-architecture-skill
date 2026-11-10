# Changelog

All notable changes to this skill are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). For a skill, the version number means:

- **Major:** a rule or convention changes, so code that followed the previous version no longer complies (a renamed folder, a reversed rule, a different message flow).
- **Minor:** new guidance that doesn't contradict the existing one (a new reference, section, rule or template).
- **Patch:** fixes and clarifications that don't change what the skill asks for (wording, broken links, examples that didn't compile).

## [Unreleased]

## [1.0.0] - 2026-11-10

### Added

- `SKILL.md` with the core rules, the layout at a glance, the workflow for each kind of change (command, query, subscriber, shared logic, crossing modules), naming conventions, a code review guide and a table of when to read each reference.
- `references/project-structure.md`: bounded contexts as independent crates, modules built around one aggregate, `<context>/<module>/<layer>` layout, the context's `shared` module and `libs/shared`, apps, dependency rules, and why module-first scales better than layer-first.
- `references/hexagonal-architecture.md`: abstractions, interfaces (Java, other languages, Rust traits), dependency inversion, ports and adapters, the domain, application and infrastructure layers, use cases, and domain services for logic shared by several use cases.
- `references/value-objects.md`: validated, immutable Value Objects, aggregate roots as containers of VOs, nullable concepts modelled inside the VO (never `Option<VO>`), and identifiers as VOs.
- `references/cqrs.md`: commands and queries, one handler per message, TypeId-based in-memory buses, use case layout, dispatching from entry points, and application services wrapping domain services so handlers never call the domain directly.
- `references/dependency-injection.md`: constructor injection with `Arc<dyn Trait>`, what each component receives, use cases with injected `EventBus`, `QueryBus` and `CommandBus`, and the composition root.
- `references/domain-events.md`: events recorded by aggregates and published by application services after persisting, idempotent subscribers in the consuming module, where event types live, and delivery guarantees.
- `references/testing.md`: hand-written mocks of the ports, Object Mothers (primitive, Value Object and aggregate mothers with `with_<field>`), what to test in each layer, adapter contract tests and end-to-end tests.
- `assets/templates/`: 30 Rust templates with placeholders for the domain building blocks, the create, change and find use cases, a subscriber, the in-memory adapter, controllers, the composition root, a repository mock, mothers and the standard use case tests.
- `README.md` and `LICENSE` (MIT).

[Unreleased]: https://github.com/otsobide/ddd-clean-architecture-skill/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/otsobide/ddd-clean-architecture-skill/releases/tag/v1.0.0
