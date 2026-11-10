# Project Structure

How the codebase is laid out: a Cargo workspace of **bounded contexts** (library crates) split into **modules**, each module a small hexagon with `domain/`, `application/` and `infrastructure/`. Things used by more than one module or context are moved to a **shared** module or crate, and **apps** are thin binaries that import contexts and expose them.

> **Golden rule: organize by business first, by layer second.** The path is always `<bounded_context>/<module>/<layer>`, never `<layer>/<module>`.

---

## Table of contents

- [Overview](#overview)
- [Bounded contexts](#bounded-contexts)
- [Modules](#modules)
- [Layers inside a module](#layers-inside-a-module)
- [The shared module of a bounded context](#the-shared-module-of-a-bounded-context)
- [Shared between bounded contexts](#shared-between-bounded-contexts)
- [Promotion example](#promotion-example)
- [Apps](#apps)
- [Dependency rules](#dependency-rules)
- [Why module/hexagonal and not hexagonal/module](#why-modulehexagonal-and-not-hexagonalmodule)
- [Tests](#tests)
- [File and module conventions](#file-and-module-conventions)
- [Checklist](#checklist)

---

## Overview

```
.
├── Cargo.toml                          # workspace: lists every crate
│
├── apps/                               # deployable binaries (entry points)
│   └── catalog_api/                    # HTTP service exposing the catalog context
│       ├── Cargo.toml                  # depends on libs/catalog + libs/shared/*
│       └── src/
│           ├── main.rs                 # bootstrap: logging, server
│           ├── lib.rs                  # composition root + routes (reused by e2e tests)
│           ├── health/
│           └── product/                # controllers for the product module
│               ├── post.rs
│               └── get.rs
│
├── libs/
│   ├── catalog/                        # BOUNDED CONTEXT (library crate)
│   │   ├── Cargo.toml                  # depends only on libs/shared/*
│   │   └── src/
│   │       ├── lib.rs                  # pub mod product; pub mod products_counter; pub mod shared;
│   │       ├── product/                # MODULE
│   │       │   ├── domain/
│   │       │   ├── application/
│   │       │   └── infrastructure/
│   │       ├── products_counter/       # MODULE
│   │       │   ├── domain/
│   │       │   ├── application/
│   │       │   └── infrastructure/
│   │       └── shared/                 # SHARED MODULE of this context
│   │           ├── domain/
│   │           └── infrastructure/
│   │
│   ├── billing/                        # another BOUNDED CONTEXT
│   │   └── src/...
│   │
│   └── shared/                         # SHARED BETWEEN CONTEXTS (imported by several contexts)
│       ├── cqrs/                       # crate: command/query buses
│       ├── domain-events/              # crate: DomainEvent, EventBus
│       ├── valueobject/                # crate: generic Value Object primitives
│       └── testing/                    # crate: primitive mothers, mocks of shared ports (dev-dependency only)
│
└── tests/
    ├── apps/catalog_api/               # end-to-end tests of the app
    └── libs/catalog/                   # unit tests, mirroring libs/catalog/src
```

The same shape repeats at every level: workspace → bounded context → module → layer → use case. Once you know one, you can predict where anything lives.

---

## Bounded contexts

A **bounded context** is an explicit boundary inside which one domain model and its language (the *ubiquitous language*) are consistent. The same word can mean different things in different contexts:

- In **catalog**, a `Product` has a name, a description, and a category.
- In **billing**, a `Product` has a price, a tax rate, and a SKU.

Each context owns its own model. No context tries to share one "universal" `Product`.

In Rust, **each bounded context is a library crate** under `libs/<context>/`.

### Each bounded context is an independent application

- **Its own crate.** It has its own `Cargo.toml`, model, persistence, and tests.
- **It depends only on `libs/shared/*`.** Never on another bounded context. Cargo enforces this: `use billing::...` does not compile unless `billing` is declared as a dependency, and declaring it is forbidden.
- **It talks to other contexts only through domain events.** Event types that cross contexts live in `libs/shared` (see [Domain Events](domain-events.md)).
- **It builds and tests on its own.** For example: `cargo test -p catalog`.
- **It can be extracted** into its own service or repository without touching any other context, because no other context imports it. Only its apps do.

Split contexts where the language changes, where the rules differ, or where a different team owns the code.

---

## Modules

A **module** is a cohesive part of a bounded context, built around **one aggregate**: `product`, `category`, `products_counter`.

Each module is a complete hexagon:

```
product/
├── domain/
│   ├── entities/            # aggregate root (product.rs) and child entities
│   ├── value_objects/       # ProductName, ProductPrice...
│   ├── repositories/        # ProductRepository trait (port)
│   ├── ports/               # other driven ports (external services), if any
│   ├── events/              # events only this module uses
│   ├── errors/              # ProductError
│   └── services/            # domain services shared by several use cases
├── application/
│   ├── create_product/      # one folder per use case (see CQRS)
│   ├── find_product/
│   └── rename_product/
└── infrastructure/
    └── persistence/
        ├── in_memory/       # InMemoryProductRepository
        └── postgres/        # PostgresProductRepository
```

**Modules never import each other.** Whatever two modules need lives in the context's [`shared` module](#the-shared-module-of-a-bounded-context).

---

## Layers inside a module

This is hexagonal architecture (ports and adapters) applied per module. See [Hexagonal Architecture](hexagonal-architecture.md) for the layers in depth, dependency inversion, use cases and domain services.

| Layer | Contains | May depend on |
|---|---|---|
| `domain/` | Aggregate roots, entities, Value Objects, repository traits and other ports, domain events, domain errors, domain services | Only domain code: its own and `shared/domain` (plus the `libs/shared` domain crates). No frameworks, no I/O |
| `application/` | Use cases: commands, queries, handlers, application services, responses, event subscribers | `domain/` |
| `infrastructure/` | Adapters: repository implementations, external API clients, bus implementations | `domain/`, `application/`, and external crates (sqlx, reqwest...) |

```
infrastructure ───► application ───► domain
      │                                ▲
      └────────────────────────────────┘
          implements the domain's ports (traits)
```

- **Ports** (traits such as `ProductRepository`) are declared in `domain/`.
- **Adapters** (`InMemoryProductRepository`, `PostgresProductRepository`) implement them in `infrastructure/`.
- Application services depend on the trait (`Arc<dyn ProductRepository>`), never on an adapter. The concrete adapter is chosen in the app's composition root.

Swapping Postgres for another database means writing a new adapter. Domain and application code don't change.

---

## The shared module of a bounded context

`libs/<context>/src/shared/` holds what **two or more modules of the same context** use. It has the same layers as any module:

```
libs/catalog/src/shared/
├── domain/
│   ├── value_objects/product_id.rs           # referenced by product and products_counter
│   └── events/product_created_event.rs       # published by product, consumed by products_counter
└── infrastructure/
    └── persistence/postgres_pool.rs          # connection pool used by every module's adapters
```

> **Rule: as soon as a second module needs something, move it to `shared` directly.** Never import it from the other module, not even "for now".

Why:

- **No module-to-module coupling.** Every module depends only on `shared`, never on a sibling. Dependencies stay flat and explicit, and no cycles can appear.
- **Modules stay removable.** Deleting or extracting a module never breaks a sibling, because no sibling imports it.
- **Clear ownership.** Something in a module belongs to that module only. Something in `shared` belongs to the context.

**Only on real need.** Things move to `shared` when a second user actually appears, never "just in case". `shared` grows only on demand.

`shared` never imports from the modules that use it.

---

## Shared between bounded contexts

`libs/shared/` contains crates **imported by several bounded contexts** (and by apps):

- **Building blocks:** CQRS buses (`cqrs`), the domain event infrastructure (`domain-events`), generic Value Object primitives (`valueobject`).
- **Cross-context contracts:** domain events published by one context and consumed by another.

It is the same rule one level up:

> **As soon as a second bounded context needs something, it moves to `libs/shared`. A bounded context never imports another bounded context.**

Each crate in `libs/shared/` follows the same layer split internally (`domain/`, `infrastructure/`). For example, the CQRS crate keeps the bus traits in `domain/` and the in-memory buses in `infrastructure/`.

Keep `libs/shared` **small**. Everything in it couples every context that imports it. It never contains business rules that belong to a single context.

---

## Promotion example

Where `ProductCreatedEvent` lives, depending on who uses it:

```
1. Only the product module uses it:
   libs/catalog/src/product/domain/events/product_created_event.rs

2. The products_counter module (same context) subscribes to it:
   ──► libs/catalog/src/shared/domain/events/product_created_event.rs

3. The billing context subscribes to it too:
   ──► libs/shared/events/src/catalog/product_created_event.rs
       (a crate imported by both catalog and billing)
```

The event only moves, it never changes. Its payload is made of primitives, so it doesn't depend on any module's Value Objects. The same path applies to any resource: Value Objects, traits, infrastructure helpers.

---

## Apps

`apps/<app>/` are **binary crates**: the deployable entry points (HTTP API, background worker, CLI).

- **They import bounded contexts and shared crates.** Nothing imports an app.
- **They contain no domain logic.** Controllers build commands or queries from the request, dispatch them, and map the result (see [CQRS](cqrs.md)).
- **They own the composition root.** `lib.rs` picks the concrete adapters (in-memory, Postgres...), builds application services and handlers, registers them on the buses, and declares the routes. `main.rs` only bootstraps (logging, configuration, server). Keeping the wiring in `lib.rs` lets end-to-end tests reuse it.
- **Controllers are grouped by module:** `apps/catalog_api/src/product/post.rs`, `get.rs`...

Each bounded context is usually exposed by its own app (`catalog` → `catalog_api`), so it can be deployed on its own. An app may import several contexts when they are deliberately deployed together.

```toml
# libs/catalog/Cargo.toml (a bounded context)
[package]
name = "catalog"
version = "0.1.0"
edition = "2021"

[dependencies]
shared-cqrs = { path = "../shared/cqrs" }
shared-domain-events = { path = "../shared/domain-events" }
shared-valueobject = { path = "../shared/valueobject" }
# Never another context: billing = { path = "../billing" }  ✗
```

```toml
# apps/catalog_api/Cargo.toml (an app)
[package]
name = "catalog-api"
version = "0.1.0"
edition = "2021"

[dependencies]
catalog = { path = "../../libs/catalog" }
shared-cqrs = { path = "../../libs/shared/cqrs" }
shared-domain-events = { path = "../../libs/shared/domain-events" }
actix-web = "4"
```

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "apps/catalog_api",
    "libs/catalog",
    "libs/billing",
    "libs/shared/cqrs",
    "libs/shared/domain-events",
    "libs/shared/valueobject",
]
```

---

## Dependency rules

```
apps/<app> ───────► libs/<context> ───────► libs/shared/*
     └────────────────────────────────────► libs/shared/*

inside libs/<context>/src/:
    <module> ───► shared                    never <module> ───► <other module>

inside every module (and every shared module / crate):
    infrastructure ───► application ───► domain
```

| Forbidden | Instead |
|---|---|
| `libs/<context>` → `libs/<other context>` | Move what both need to `libs/shared`; communicate through events |
| `<module>` → `<other module>` | Move what both need to the context's `shared` |
| `shared` → a module that uses it | `shared` is a leaf: modules depend on it, never the opposite |
| `libs/shared` → `libs/<context>` | Shared crates know no context |
| `libs/*` → `apps/*` | Apps are the outermost layer; nothing imports them |
| `domain` → `application` / `infrastructure` | Declare a port (trait) in `domain`, implement it in `infrastructure` |

Crate boundaries are enforced by Cargo: an undeclared dependency does not compile. Module boundaries inside a crate are a convention, so keep them visible in code review: an import of `crate::<other_module>::...` inside a module is always a mistake.

---

## Why module/hexagonal and not hexagonal/module

```
✅ module first                        ❌ layer first
libs/catalog/src/                      libs/catalog/src/
├── product/                           ├── domain/
│   ├── domain/                        │   ├── product/
│   ├── application/                   │   ├── category/
│   └── infrastructure/                │   └── products_counter/
├── category/                          ├── application/
│   ├── domain/                        │   ├── product/
│   ├── application/                   │   ├── category/
│   └── infrastructure/                │   └── products_counter/
├── products_counter/                  └── infrastructure/
│   └── ...                                ├── product/
└── shared/                                ├── category/
                                           └── products_counter/
```

Module first scales better:

- **Cohesion.** Everything about `product` is in one folder. A change to a feature touches one subtree, not three distant ones.
- **Additive growth.** A new module is a new folder, and existing folders don't grow. With layer first, every new module makes `domain/`, `application/` and `infrastructure/` bigger, until each holds hundreds of files.
- **Visible boundaries.** With module first, `crate::category::...` inside `product/` stands out as a cross-module import. With layer first, `crate::domain::category` from `crate::domain::product` looks harmless, and coupling creeps in.
- **Cheap extraction and deletion.** Moving a module to another context or crate, or deleting it, means moving or deleting one folder.
- **Ownership and parallel work.** Teams own modules. Work on different modules rarely touches the same folders, so merge conflicts are rare.
- **Screaming architecture.** The top-level folders say what the system does (`product`, `category`), not which layers or frameworks it uses.

---

## Tests

Tests live outside the crates, in `tests/`, mirroring the source tree:

```
tests/
├── libs/catalog/                       # unit tests of the catalog context
│   ├── tests.rs                        # test crate root
│   └── src/
│       ├── mocks/                      # hand-written mocks of this context's ports
│       ├── product/
│       │   ├── domain/                 # Object Mothers, VO and aggregate tests
│       │   ├── application/create_product/product_creator_tests.rs
│       │   └── infrastructure/         # adapter contract tests
│       └── products_counter/...
└── apps/catalog_api/                   # end-to-end tests through the app's routes
    ├── tests.rs
    └── src/product/create_product_test.rs
```

Each test crate is registered from the crate it tests:

```toml
# libs/catalog/Cargo.toml
[[test]]
name = "catalog"
path = "../../tests/libs/catalog/tests.rs"
```

Because the tests are an external crate, they can only reach the context's public API, just like a real consumer. See [Testing](testing.md) for mocks, Object Mothers and what to test in each layer.

---

## File and module conventions

- **Folders and files** use `snake_case`. **Package names** use `kebab-case` (`shared-cqrs`) and are imported as `snake_case` (`shared_cqrs`).
- **One public type per file**, and the file name is the type name in `snake_case`: `product_creator.rs` → `ProductCreator`.
- **`mod.rs` files only declare submodules.** No logic.
- **Use case folders** are `<verb>_<noun>` (`create_product/`). **Subscriber folders** are `<reaction>_on_<noun>_<past>` (`increment_products_counter_on_product_created/`).

---

## Checklist

- [ ] Each bounded context is its own crate under `libs/<context>/`, depending only on `libs/shared/*`
- [ ] Inside a context, code is organized as `<module>/{domain,application,infrastructure}`
- [ ] Each module is built around one aggregate
- [ ] No module imports another module; shared resources live in the context's `shared/`
- [ ] No context imports another context; cross-context resources live in `libs/shared/`
- [ ] Resources move to `shared` as soon as a second user appears, never before
- [ ] `domain/` depends on nothing outside the domain; ports are traits in `domain/`, adapters in `infrastructure/`
- [ ] Apps only import contexts and shared crates, hold the composition root, and contain no domain logic
- [ ] Tests live in `tests/libs/<context>/` and `tests/apps/<app>/`, mirroring the source tree
