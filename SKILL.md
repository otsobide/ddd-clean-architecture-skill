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

- [Value Objects](references/value-objects.md): validated, immutable domain primitives. Nullable concepts are modelled inside the VO, never as `Option<VO>` in aggregates.
- [CQRS](references/cqrs.md): commands vs queries, handlers, buses, use case layout, and how entry points dispatch.
- [Domain Events](references/domain-events.md): facts recorded by aggregates, published after persisting, and handled by idempotent subscribers in the consuming context.
