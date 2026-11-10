---
name: ddd-clean-architecture
description: "Domain-Driven Design with hexagonal architecture, CQRS and domain events for backend services, with examples in Rust. Lays code out as bounded contexts and modules with domain, application and infrastructure layers; models aggregates and Value Objects; declares ports as traits and implements adapters in infrastructure; runs use cases through command, query and event buses; wires dependencies in a composition root; and tests with hand-written mocks and Object Mothers. Use this skill whenever you create, extend, refactor or review a DDD or clean-architecture codebase: adding a bounded context, module, aggregate, Value Object, use case, command, query, handler, domain event, subscriber, repository or adapter, deciding where a piece of code belongs, or writing tests for any of them. Also use it when the user mentions DDD, hexagonal architecture, ports and adapters, clean architecture, CQRS, event bus, aggregate root, value object, bounded context or Object Mother, even if they don't name this skill."
---

# DDD & Clean Architecture

This skill builds backend services with Domain-Driven Design and hexagonal architecture:

- The code is split into **bounded contexts** (one crate each) and **modules** (one aggregate each). Every module has three layers: `domain`, `application` and `infrastructure`.
- The **domain** holds the business model (aggregates, Value Objects, events) and declares the **ports** (traits) it needs from the outside. **Infrastructure** implements them as adapters.
- Every use case is a **command** or a **query**, dispatched through a bus to exactly one handler. State changes are announced as **domain events**.
- Dependencies are injected through constructors and wired in one **composition root** per app. Tests replace infrastructure with **mocks** and build their data with **Object Mothers**.

Code examples are in Rust (Rust 2021, `async-trait`, `thiserror`, `tokio`, `actix-web`). If the project uses another language, apply the same structure and rules with that language's idioms.

## Core rules

These rules are not negotiable. Each one is explained in a reference file.

1. **Organize by business first, by layer second.** Paths are `<context>/<module>/<layer>`, never `<layer>/<module>` ([project structure](references/project-structure.md)).
2. **Dependencies point inward.** The domain depends on nothing. It declares ports as traits; infrastructure implements them. Application services depend on `Arc<dyn Port>`, never on an adapter ([hexagonal architecture](references/hexagonal-architecture.md)).
3. **Aggregates hold only Value Objects, never `Option<VO>`.** VOs validate in their constructor. An optional concept is a VO that accepts `Option<T>` ([Value Objects](references/value-objects.md)).
4. **A use case is either a command or a query.** Commands return no data; the caller generates the ids. Queries have no side effects and return primitive DTOs. Each message has exactly one handler ([CQRS](references/cqrs.md)).
5. **Handler → application service → domain.** Handlers and subscribers only call application services. An application service never calls another one: logic that several use cases share becomes a domain service, and each use case wraps it ([hexagonal architecture](references/hexagonal-architecture.md#when-a-use-case-needs-another-use-case-domain-services), [CQRS](references/cqrs.md#handlers-never-call-domain-services)).
6. **Aggregates record events; only application services publish them, after persisting.** Domain code never touches the `EventBus` ([domain events](references/domain-events.md)).
7. **Modules and contexts never import each other.** They communicate through the `QueryBus`, the `CommandBus` or events. Anything a second module needs moves to the context's `shared` module, and anything a second context needs moves to `libs/shared`, as soon as that second user appears and never before ([project structure](references/project-structure.md), [dependency injection](references/dependency-injection.md)).
8. **Constructor injection, one composition root per app.** No service builds its own collaborators. No globals, no service locators ([dependency injection](references/dependency-injection.md)).
9. **Mock only the ports.** Domain and application code under test is real, and data comes from Object Mothers ([testing](references/testing.md)).

## Layout at a glance

```
apps/<app>/src/lib.rs                       composition root + routes
libs/<context>/src/
├── <module>/
│   ├── domain/
│   │   ├── entities/                       aggregate root
│   │   ├── value_objects/
│   │   ├── repositories/                   repository traits (ports)
│   │   ├── ports/                          other driven ports
│   │   ├── events/
│   │   ├── errors/
│   │   └── services/                       domain services
│   ├── application/<verb>_<noun>/          one folder per use case
│   └── infrastructure/persistence/<tech>/  adapters
└── shared/                                 what two or more modules of this context use
libs/shared/                                what two or more contexts use (cqrs, domain-events, valueobject, testing)
tests/libs/<context>/   tests/apps/<app>/   tests mirroring the source tree
```

```
infrastructure ──► application ──► domain
       └─────── implements ──────────▲
```

## Workflow

### Before writing code

1. **Read the existing layout.** Find the contexts, the modules, the shared crates and the composition root. If the project already has its own conventions (names, folders, error style), follow them. This skill's conventions are the default for anything the project doesn't define.
2. **Place the change.** Decide which bounded context and which module (aggregate) it belongs to. If it touches two modules, decide which one owns it. The other module takes part through a bus.
3. **Classify it.** It may be a command (changes state), a query (reads), a reaction to an event (subscriber), a new domain rule (aggregate or VO), or a new adapter.

### Adding a command use case

Work from the inside out. Start each file from its template in [assets/templates](assets/templates/README.md) when there is one.

1. **Value Objects** for every new input, each validating in its constructor ([Value Objects](references/value-objects.md)).
2. **Aggregate behaviour.** A method that enforces the rule, changes state and records a past-tense event; a no-op records nothing ([domain events](references/domain-events.md#the-aggregate-records-its-events)).
3. **The event**, with a primitive payload and `EVENT_NAME` = `<context>.<aggregate>.<past_verb>` ([domain events](references/domain-events.md#defining-an-event)).
4. **Port methods** the use case needs on the repository trait, in domain language ([hexagonal architecture](references/hexagonal-architecture.md#rust-traits)).
5. **The application service** `<Noun><Verb>er::execute`: load, call the aggregate, persist, then publish `pull_domain_events()` ([hexagonal architecture](references/hexagonal-architecture.md#use-cases)).
6. **The command and its handler** in `application/<verb>_<noun>/`. The handler only delegates ([CQRS](references/cqrs.md#command-use-case-example)).
7. **The adapter methods** in every adapter of the port: in-memory and production ([hexagonal architecture](references/hexagonal-architecture.md#the-port-belongs-to-its-consumer)).
8. **Wiring.** Build the service and register the handler in the composition root ([dependency injection](references/dependency-injection.md#the-composition-root)).
9. **The entry point.** The controller builds VOs (400 on invalid input), dispatches, and maps domain errors to statuses ([CQRS](references/cqrs.md#dispatching-from-an-entry-point)).
10. **Tests.** Write unit tests of the application service with mocks and mothers, contract tests for the new adapter methods, and an end-to-end test of the endpoint ([testing](references/testing.md)).

### Adding a query use case

Follow the same steps, except that there is no event. The application service returns the aggregate, and the handler maps it to a primitive response DTO. A query service never receives the `EventBus` or the `CommandBus` ([CQRS](references/cqrs.md#query-use-case-example)).

### Reacting to an event

The subscriber lives in the **consuming** module, is named `<Reaction>On<Noun><Past>`, delegates to an application service of its own module, and is idempotent. If another module needs the event type, move it to `shared` ([domain events](references/domain-events.md#subscribers)).

### Reusing logic between use cases

As soon as a second use case needs logic that an application service already has, move that logic to `domain/services/` as a domain service. Each use case's application service then wraps it: it calls the domain service and, when the domain service finishes, publishes the events ([hexagonal architecture](references/hexagonal-architecture.md#when-a-use-case-needs-another-use-case-domain-services)).

### Talking to another module or context

Inject the bus into the application service, never into the handler:

- **Read** another module's data with a query through the `QueryBus`.
- **Make another module act** with an event (preferred) or a command through the `CommandBus`.

The messages and the VOs they carry live in `shared` ([dependency injection](references/dependency-injection.md#buses-are-dependencies-too)).

## Naming conventions

| Concept | Pattern | Example | Location |
|---|---|---|---|
| Aggregate root | `Noun` | `Product` | `domain/entities/` |
| Value Object | `NounConcept` | `ProductName` | `domain/value_objects/` |
| Repository port | `NounRepository` | `ProductRepository` | `domain/repositories/` |
| Adapter | `TechNounRepository` | `PostgresProductRepository`, `InMemoryProductRepository` | `infrastructure/persistence/<tech>/` |
| Domain error | `NounError` | `ProductError` | `domain/errors/` |
| Domain event | `NounPastEvent` | `ProductRenamedEvent` (`catalog.product.renamed`) | `domain/events/` |
| Domain service | `NounVerber` | `ProductFinder` (method named after the operation) | `domain/services/` |
| Use case folder | `verb_noun` | `rename_product/` | `application/` |
| Application service | `NounVerber` | `ProductRenamer::execute` | use case folder |
| Command / handler | `VerbNounCommand[Handler]` | `RenameProductCommandHandler` | use case folder |
| Query / handler / response | `VerbNounQuery[Handler]`, `VerbNounResponse` | `FindProductResponse` | use case folder |
| Subscriber | `ReactionOnNounPast` | `IncrementProductsCounterOnProductCreated` | `application/<reaction>_on_<noun>_<past>/` |
| Mock | `PortMock` | `ProductRepositoryMock` | `tests/libs/<context>/src/mocks/` |
| Mother | `TypeMother` | `ProductMother`, `ProductNameMother` | `mothers/`, next to the mirrored type |

Files contain one public type each and use `snake_case` names. `mod.rs` files only declare submodules.

## Reviewing code

Flag these, in this order of severity:

- Domain code importing a framework, an I/O crate, an adapter or a bus.
- A module or context importing another one, instead of using a bus or `shared`.
- `Option<VO>` or raw primitives inside an aggregate, or a VO that can be built invalid.
- A command that returns data, or a query that writes or publishes.
- A handler or subscriber that holds logic or calls a domain service. An application service that calls another one.
- Events built outside the aggregate, published before persisting, or published by domain code.
- A port that returns technology types, or a service that depends on an adapter or builds its own dependencies.
- Tests that mock domain code, or literal values that the test is not about.

Each reference ends with a checklist and an anti-pattern table to use for a detailed review.

## Templates

[assets/templates](assets/templates/README.md) has ready-to-copy files with placeholders (`{{Aggregate}}`, `{{concept}}`, `{{verb}}`...). There are templates for every domain building block, the create, change and find use cases, a subscriber, the in-memory adapter, controllers, the composition root, a repository mock, mothers and the standard use case tests. Copy each one to the path in its first line, replace the placeholders, and resolve the `TODO`s.

## References

Read the file for the task at hand. Each one opens with a table of contents.

| File | Read it when |
|---|---|
| [project-structure.md](references/project-structure.md) | Creating a project, context, module or app; deciding where a file or shared resource belongs; editing `Cargo.toml` |
| [hexagonal-architecture.md](references/hexagonal-architecture.md) | Defining ports and adapters, deciding which layer code belongs to, writing a use case, extracting a domain service |
| [value-objects.md](references/value-objects.md) | Creating or changing a Value Object, an aggregate's fields, or an optional concept |
| [cqrs.md](references/cqrs.md) | Adding a command or query, its handler or response, a bus, or an entry point that dispatches |
| [dependency-injection.md](references/dependency-injection.md) | Wiring the composition root, injecting buses into a use case, crossing modules |
| [domain-events.md](references/domain-events.md) | Recording or publishing events, writing a subscriber, delivery guarantees |
| [testing.md](references/testing.md) | Writing mocks, Object Mothers, unit, contract or end-to-end tests |
