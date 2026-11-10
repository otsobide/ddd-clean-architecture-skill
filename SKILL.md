---
name: ddd-clean-architecture
description: TODO — what this skill does and when it should trigger.
---

# DDD & Clean Architecture

## When to use this skill

## Principles

## Workflow

## Examples

## References

- [Project Structure](references/project-structure.md): workspace layout, bounded contexts as independent crates, modules as `<module>/{domain,application,infrastructure}`, shared modules and crates, apps, and the dependency rules.
- [Hexagonal Architecture](references/hexagonal-architecture.md): abstractions, interfaces (Java, other languages, Rust traits), dependency inversion, ports and adapters, the domain / application / infrastructure layers, use cases, and domain services for logic shared by use cases.
- [Value Objects](references/value-objects.md): validated, immutable domain primitives. Nullable concepts are modelled inside the VO, never as `Option<VO>` in aggregates.
- [CQRS](references/cqrs.md): commands vs queries, handlers, buses, use case layout, and how entry points dispatch.
- [Dependency Injection](references/dependency-injection.md): constructor injection with `Arc<dyn Trait>`, what each component receives, use cases with injected `EventBus` / `QueryBus` / `CommandBus`, and the composition root.
- [Domain Events](references/domain-events.md): facts recorded by aggregates, published after persisting, and handled by idempotent subscribers in the consuming context.
- [Testing](references/testing.md): hand-written mocks of the ports replacing infrastructure, Object Mothers (primitive, Value Object and aggregate mothers with `with_<field>`), what to test in each layer, adapter contract tests, and end-to-end tests.
