# Code templates

Ready-to-copy Rust files for the most common building blocks of a module. Each one follows the conventions of the references, so generated code has the same shape every time.

Rendered with the example values below (`Product`, `name`, `rename`), the templates produce a workspace that compiles without warnings and whose tests pass. That includes the unit tests of the change use case, a subscriber and an end-to-end test of the three endpoints.

## How to use them

1. **Pick the templates** for the task (see the index below). A new module usually needs every domain template. A new use case needs only its application folder, its tests, and the wiring.
2. **Copy each one to the path in its first line** (`// Template → ...`) and delete that line.
3. **Replace every placeholder** with the same values in every file of the task.
4. **Resolve each `TODO`**: the domain's limits, what blank means for an optional value, production adapters.
5. **Declare the new modules** in the parent `mod.rs` files, and **wire** the services and handlers in the composition root (`app/composition_root.rs` shows the order).
6. **Adapt to what the task really needs.** The templates have one Value Object field (`{{concept}}`). Add one field, one VO and one mother per extra concept. Remove the parts you don't need.

The imports assume the shared crates `shared_cqrs`, `shared_domain_events`, `shared_valueobject` and `shared_testing`, laid out as in the references. If the project already names or organizes them differently, follow the project.

## Placeholders

| Placeholder | Meaning | Example |
|---|---|---|
| `{{context}}` | Bounded context crate | `catalog` |
| `{{app}}` | App crate | `catalog_api` |
| `{{Aggregate}}` / `{{aggregate}}` / `{{aggregates}}` | Aggregate root: PascalCase / snake_case / plural for routes and collections | `Product` / `product` / `products` |
| `{{Concept}}` / `{{concept}}` | A Value Object field of the aggregate; the VO is `{{Aggregate}}{{Concept}}` | `Name` / `name` → `ProductName` |
| `{{Verb}}` / `{{verb}}` | Imperative verb of a change use case, also the aggregate's behaviour method | `Rename` / `rename` |
| `{{Verber}}` / `{{verber}}` | Agent noun of that verb, for the application service `{{Aggregate}}{{Verber}}` | `Renamer` / `renamer` |
| `{{Past}}` / `{{past}}` | Past tense of that verb, for the event `{{Aggregate}}{{Past}}Event` | `Renamed` / `renamed` |
| `{{Module}}` / `{{module}}` | Module that reacts to an event (subscriber only) | `ProductsCounter` / `products_counter` |
| `{{Reaction}}` / `{{reaction}}` | What the subscriber does (subscriber only) | `IncrementProductsCounter` / `increment_products_counter` |
| `{{ReactionService}}` / `{{reaction_service}}` | Application service the subscriber calls (subscriber only) | `ProductsCounterIncrementer` / `products_counter_incrementer` |

## Index

### Domain (`libs/{{context}}/src/{{aggregate}}/domain/`)

| Template | Produces | Reference |
|---|---|---|
| `domain/identifier.rs` | `{{Aggregate}}Id`: UUID identifier VO | [Value Objects](../../references/value-objects.md#identifiers-are-value-objects-too) |
| `domain/value_object.rs` | `{{Aggregate}}{{Concept}}`: validated string VO | [Value Objects](../../references/value-objects.md#basic-vo-validated-newtype) |
| `domain/nullable_value_object.rs` | `{{Aggregate}}{{Concept}}` for an optional concept | [Value Objects](../../references/value-objects.md#nullable-concepts-null-lives-inside-the-vo-never-around-it) |
| `domain/aggregate.rs` | `{{Aggregate}}`: `create`, `reconstitute`, a behaviour that records an event, `pull_domain_events` | [Domain Events](../../references/domain-events.md#the-aggregate-records-its-events) |
| `domain/created_event.rs` | `{{Aggregate}}CreatedEvent` | [Domain Events](../../references/domain-events.md#defining-an-event) |
| `domain/changed_event.rs` | `{{Aggregate}}{{Past}}Event`, with previous and new values | [Domain Events](../../references/domain-events.md#defining-an-event) |
| `domain/error.rs` | `{{Aggregate}}Error` | [Hexagonal Architecture](../../references/hexagonal-architecture.md#rust-traits) |
| `domain/repository.rs` | `{{Aggregate}}Repository` port | [Hexagonal Architecture](../../references/hexagonal-architecture.md#rust-traits) |
| `domain/finder.rs` | `{{Aggregate}}Finder` domain service: find or `NotFound` | [Hexagonal Architecture](../../references/hexagonal-architecture.md#when-a-use-case-needs-another-use-case-domain-services) |

### Application (`libs/{{context}}/src/<module>/application/`)

| Template | Produces | Reference |
|---|---|---|
| `application/create/` | `Create{{Aggregate}}Command`, its handler, `{{Aggregate}}Creator` | [CQRS](../../references/cqrs.md#command-use-case-example) |
| `application/change/` | `{{Verb}}{{Aggregate}}Command`, its handler, `{{Aggregate}}{{Verber}}` (loads through the domain finder) | [Hexagonal Architecture](../../references/hexagonal-architecture.md#use-cases) |
| `application/find/` | `Find{{Aggregate}}Query`, its handler, `Find{{Aggregate}}Response`, the application `{{Aggregate}}Finder` wrapping the domain one | [CQRS](../../references/cqrs.md#query-use-case-example) |
| `application/subscriber.rs` | `{{Reaction}}On{{Aggregate}}{{Past}}`, in the consuming module | [Domain Events](../../references/domain-events.md#subscribers) |

### Infrastructure and app

| Template | Produces | Reference |
|---|---|---|
| `infrastructure/in_memory_repository.rs` | `InMemory{{Aggregate}}Repository` | [Hexagonal Architecture](../../references/hexagonal-architecture.md#rust-traits) |
| `app/post.rs`, `app/put.rs`, `app/get.rs` | Controllers: build VOs (400), dispatch, map domain errors to statuses | [CQRS](../../references/cqrs.md#dispatching-from-an-entry-point) |
| `app/composition_root.rs` | `compose()` and `configure()` for the app's `lib.rs` | [Dependency Injection](../../references/dependency-injection.md#the-composition-root) |

### Tests (`tests/libs/{{context}}/src/`)

| Template | Produces | Reference |
|---|---|---|
| `tests/repository_mock.rs` | `{{Aggregate}}RepositoryMock`: stub and spy | [Testing](../../references/testing.md#mocks) |
| `tests/identifier_mother.rs`, `tests/value_object_mother.rs` | VO mothers: `create`, `random` | [Testing](../../references/testing.md#value-object-mothers) |
| `tests/aggregate_mother.rs` | `{{Aggregate}}Mother`: `create`, `random`, `with_id`, `with_{{concept}}` | [Testing](../../references/testing.md#aggregate-mothers) |
| `tests/change_service_tests.rs` | The standard tests of a change use case | [Testing](../../references/testing.md#unit-tests-of-application-services) |
