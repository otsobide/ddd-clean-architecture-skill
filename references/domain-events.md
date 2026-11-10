# Domain Events

A **domain event** is an immutable record of something meaningful that **already happened** in the domain: `ProductCreatedEvent`, `ProductRenamedEvent`, `OrderPaidEvent`.

Domain events decouple the code that changes state from the code that reacts to that change. The use case that creates a product does not know, and must not know, that a counter gets incremented, a search index gets updated, or an email gets sent. It records a fact, and whoever cares reacts.

> **Golden rule: an event is a fact, not a wish.** The aggregate records it when its state changes. The application service publishes it **only after the change has been persisted**. Subscribers can never undo or reject it.

---

## Table of contents

- [Why domain events](#why-domain-events)
- [Core concepts](#core-concepts)
- [Events vs commands](#events-vs-commands)
- [Rules](#rules)
- [Flow](#flow)
- [Layout and naming](#layout-and-naming)
- [Building blocks in Rust](#building-blocks-in-rust)
- [In-memory event bus](#in-memory-event-bus)
- [Defining an event](#defining-an-event)
- [The aggregate records its events](#the-aggregate-records-its-events)
- [Publishing from the application service](#publishing-from-the-application-service)
- [Domain services never publish: the application wrapper does](#domain-services-never-publish-the-application-wrapper-does)
- [Subscribers](#subscribers)
- [Wiring: the composition root](#wiring-the-composition-root)
- [Delivery guarantees](#delivery-guarantees)
- [Testing](#testing)
- [Anti-patterns](#anti-patterns)
- [Checklist](#checklist)

---

## Why domain events

- **Decoupled side effects.** Without events, `ProductCreator` would have to call the counter, the search indexer, and the mailer directly, and it would depend on every module that cares about new products. With events, it depends on none of them.
- **Communication between bounded contexts.** Contexts react to each other's events instead of importing each other's internals.
- **Extensibility.** A new reaction is a new subscriber. The publishing code never changes, and existing subscribers are unaffected.
- **Testability.** The publisher is tested by asserting which events it published. Each subscriber is tested in isolation by feeding it an event.

---

## Core concepts

| Concept | Role |
|---|---|
| **Domain event** | Immutable fact, named in the past tense, with a primitive payload |
| **Event metadata** | `event_id`, `aggregate_id`, `occurred_on`, plus a stable `event_name` |
| **Aggregate** | Records events as a consequence of its own state changes |
| **Application service** | Persists the aggregate, then pulls its recorded events and publishes them |
| **Event bus** | Delivers each published event to every subscriber of that event type |
| **Subscriber** | Reacts to one event type by delegating to an application service |

---

## Events vs commands

| | Command | Domain event |
|---|---|---|
| Meaning | Intent: "please do this" | Fact: "this happened" |
| Tense | Imperative: `CreateProductCommand` | Past: `ProductCreatedEvent` |
| Receivers | Exactly one handler | Zero, one, or many subscribers |
| Can be rejected | Yes | No, it already happened |
| Who creates it | An entry point | An aggregate |
| Payload | Value Objects | Primitives |

---

## Rules

1. **Past tense names.** `ProductCreatedEvent`, never `CreateProductEvent`.
2. **Immutable.** Fields are set once at construction.
3. **The aggregate records its own events.** Each state-changing method of the aggregate records the event describing the change. If nothing changed, nothing is recorded.
4. **Persist first, publish second.** The application service saves the aggregate, and only then pulls and publishes its events. If the save fails, nothing is published.
5. **Primitive payload.** Events carry `String`, numbers, `Option<String>`... never Value Objects or entities. Events cross bounded-context boundaries and may be serialized, so consumers must not depend on the producer's domain types.
6. **Self-contained payload.** The event carries everything a subscriber typically needs, so subscribers don't have to query back the producer.
7. **Stable event name.** Each event has a constant name `<context>.<aggregate>.<past_verb>` (for example `catalog.product.created`). It identifies the event outside the process: logs, brokers, stored events.
8. **Reconstitution records nothing.** Rebuilding an aggregate from storage must not record events. Only real state changes do.
9. **Subscribers live in the consuming module.** The module (or context) that reacts owns the subscriber, never the one that publishes.
10. **Subscribers are thin.** They translate the event into a call to an application service (never a domain service), the same way command handlers do.
11. **Subscribers are idempotent.** The same event may be delivered more than once. Processing it twice must have the same effect as processing it once.
12. **A subscriber failure never fails the publisher.** The fact already happened. Delivery problems are the bus's responsibility: log, retry, or use an outbox.
13. **Events are not for invariants.** Rules that must hold atomically belong in the aggregate. Events drive side effects and eventual consistency.
14. **The event type lives where all its users can import it.** That is the publishing module, then the context's `shared` module, then `libs/shared`, as more modules and contexts start using it.
15. **Only the application layer publishes.** The `EventBus` is injected only into application services. Aggregates and domain services record events and never publish them. The application calls the domain and, when the domain finishes, publishes what it recorded.

---

## Flow

```
CreateProductCommandHandler
  └── ProductCreator::execute(id, name)
        ├── Product::create(id, name)
        │     └── records ProductCreatedEvent        (inside the aggregate)
        ├── ProductRepository::save(&product)        (persist first)
        └── EventBus::publish(product.pull_domain_events())   (publish second)
              │  routes by event type
              ├── IncrementProductsCounterOnProductCreated::on(&event)
              │     └── ProductsCounterIncrementer::execute(...)
              └── IndexProductOnProductCreated::on(&event)
                    └── ProductIndexer::execute(...)
```

---

## Layout and naming

```
libs/catalog/src/                                # bounded context
├── shared/
│   └── domain/events/
│       └── product_created_event.rs             # used by two modules: lives in the context's shared module
├── product/                                     # publishing module
│   └── domain/
│       ├── entities/product.rs                  # aggregate: records events
│       └── events/product_renamed_event.rs      # used only by this module (for now)
└── products_counter/                            # consuming module
    └── application/
        └── increment_products_counter_on_product_created/
            ├── increment_products_counter_on_product_created.rs   # subscriber
            └── products_counter_incrementer.rs                    # application service
```

Where an event type lives depends on who uses it:

- **Only the publishing module:** in that module's `domain/events/`.
- **Another module of the same context subscribes:** it moves to the context's `shared/domain/events/`.
- **Another bounded context subscribes:** it moves to `libs/shared`.

See [Project Structure](project-structure.md) for the rule behind this.

| Concept | Pattern | Example |
|---|---|---|
| Domain event | `NounPastEvent` | `ProductCreatedEvent` |
| Event name | `<context>.<aggregate>.<past_verb>` | `catalog.product.created` |
| Subscriber | `ReactionOnNounPast` | `IncrementProductsCounterOnProductCreated` |
| Subscriber folder | `<reaction>_on_<noun>_<past>` | `increment_products_counter_on_product_created/` |
| Application service called by the subscriber | `NounVerber` | `ProductsCounterIncrementer` |

---

## Building blocks in Rust

The event building blocks live in a shared crate: traits in `domain/`, implementations in `infrastructure/`.

```rust
// shared domain events: domain/domain_event.rs

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::time::SystemTime;

/// Type-erasure helper so events can travel as `Box<dyn DomainEvent>`.
/// Implemented automatically for every `Any + Send + Sync` type.
pub trait AnyDomainEvent: Any + Send + Sync {
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
    fn domain_event_type_id(&self) -> TypeId;
}

impl<T: Any + Send + Sync> AnyDomainEvent for T {
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn domain_event_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

pub trait DomainEvent: AnyDomainEvent {
    fn event_name(&self) -> &'static str;
    fn event_id(&self) -> &str;
    fn aggregate_id(&self) -> &str;
    fn occurred_on(&self) -> SystemTime;
}
```

```rust
// shared domain events: domain/domain_event_metadata.rs

use std::time::SystemTime;
use uuid::Uuid;

/// Metadata shared by every event. Concrete events embed it and delegate to it.
pub struct DomainEventMetadata {
    pub event_id: String,
    pub aggregate_id: String,
    pub occurred_on: SystemTime,
}

impl DomainEventMetadata {
    /// New event happening now.
    pub fn new(aggregate_id: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            aggregate_id: aggregate_id.into(),
            occurred_on: SystemTime::now(),
        }
    }

    /// Existing event rebuilt from storage or from a broker message.
    pub fn from_primitives(
        event_id: impl Into<String>,
        aggregate_id: impl Into<String>,
        occurred_on: SystemTime,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            aggregate_id: aggregate_id.into(),
            occurred_on,
        }
    }
}
```

```rust
// shared domain events: domain/domain_event_subscriber.rs

use async_trait::async_trait;

/// Reacts to exactly one event type `E`.
#[async_trait]
pub trait DomainEventSubscriber<E: DomainEvent>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn on(&self, event: &E) -> Result<(), Self::Error>;
}
```

```rust
// shared domain events: domain/event_bus.rs

use async_trait::async_trait;

#[async_trait]
pub trait EventBus: Send + Sync {
    /// Delivers every event to all of its subscribers.
    ///
    /// Never fails the caller: the facts already happened. Delivery problems
    /// (failing subscribers, unreachable broker) are handled by the implementation.
    async fn publish(&self, events: Vec<Box<dyn DomainEvent>>);
}
```

---

## In-memory event bus

The bus keeps a list of type-erased subscriber functions per event `TypeId`. Each event is wrapped in an `Arc` so all of its subscribers can share it.

```rust
// shared domain events: infrastructure/in_memory/in_memory_event_bus.rs

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tracing::error;

type SubscriberFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
type SubscriberFn = Arc<dyn Fn(Arc<dyn Any + Send + Sync>) -> SubscriberFuture + Send + Sync>;

pub struct InMemoryEventBus {
    subscribers: RwLock<HashMap<TypeId, Vec<SubscriberFn>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self { subscribers: RwLock::new(HashMap::new()) }
    }

    /// Subscribes through `&self`, so the bus can be shared (`Arc`) and injected into
    /// services before its subscribers are registered (see Dependency Injection).
    pub fn subscribe<E, S>(&self, subscriber: S)
    where
        E: DomainEvent,
        S: DomainEventSubscriber<E> + 'static,
    {
        let subscriber = Arc::new(subscriber);
        self.subscribers
            .write()
            .expect("event bus lock poisoned")
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Arc::new(move |event| {
                let subscriber = Arc::clone(&subscriber);
                Box::pin(async move {
                    let event = event
                        .downcast::<E>()
                        .expect("the bus routes each event by its own TypeId");

                    if let Err(failure) = subscriber.on(&event).await {
                        error!(
                            event_name = event.event_name(),
                            event_id = event.event_id(),
                            error = %failure,
                            "Domain event subscriber failed"
                        );
                    }
                })
            }));
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, events: Vec<Box<dyn DomainEvent>>) {
        for event in events {
            let event: Arc<dyn DomainEvent> = Arc::from(event);

            // Call through `as_ref()` (see the pitfall below), and clone the subscribers
            // out of the lock so it is released before awaiting them.
            let subscribers: Vec<SubscriberFn> = self
                .subscribers
                .read()
                .expect("event bus lock poisoned")
                .get(&event.as_ref().domain_event_type_id())
                .cloned()
                .unwrap_or_default();

            for subscriber in subscribers {
                subscriber(Arc::clone(&event).into_any_arc()).await;
            }
        }
    }
}
```

The in-memory bus is in-process and sequential: `publish` returns after every subscriber has run. A broker-backed implementation (RabbitMQ, Kafka, SQS...) implements the same trait, so domain and application code don't change.

> **Pitfall.** Never call `domain_event_type_id()` (or `Any::type_id()`) directly on a `Box<dyn DomainEvent>`, an `Arc<dyn DomainEvent>`, or a `&Box<dyn DomainEvent>`. The blanket `impl<T: Any + Send + Sync> AnyDomainEvent for T` also covers the smart pointer, so method resolution picks the pointer's impl and returns its `TypeId`. No subscriber ever matches, and nothing fails loudly. Always go through `as_ref()` or `&**event`.

---

## Defining an event

```rust
// libs/catalog/src/shared/domain/events/product_created_event.rs

use std::time::SystemTime;

pub struct ProductCreatedEvent {
    metadata: DomainEventMetadata,
    pub name: String, // primitive payload
}

impl ProductCreatedEvent {
    pub const EVENT_NAME: &'static str = "catalog.product.created";

    pub fn new(product_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            metadata: DomainEventMetadata::new(product_id),
            name: name.into(),
        }
    }
}

impl DomainEvent for ProductCreatedEvent {
    fn event_name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn event_id(&self) -> &str {
        &self.metadata.event_id
    }

    fn aggregate_id(&self) -> &str {
        &self.metadata.aggregate_id
    }

    fn occurred_on(&self) -> SystemTime {
        self.metadata.occurred_on
    }
}
```

Both the constructor and the payload use primitives. The event depends on no module's Value Objects, so it can move to a shared module unchanged.

An event about a change carries both sides of it when subscribers need them:

```rust
pub struct ProductRenamedEvent {
    metadata: DomainEventMetadata,
    pub previous_name: String,
    pub name: String,
}
```

---

## The aggregate records its events

The aggregate keeps a private list of recorded events. Every state-changing method records the event that describes the change. The application service later takes them with `pull_domain_events()`.

```rust
// libs/catalog/src/product/domain/entities/product.rs

pub struct Product {
    id: ProductId,
    name: ProductName,
    domain_events: Vec<Box<dyn DomainEvent>>,
}

impl Product {
    /// A brand-new product: records `ProductCreatedEvent`.
    pub fn create(id: ProductId, name: ProductName) -> Self {
        let mut product = Self::reconstitute(id, name);
        product.record(ProductCreatedEvent::new(product.id.value(), product.name.value()));
        product
    }

    /// An existing product rebuilt from storage: records nothing.
    pub fn reconstitute(id: ProductId, name: ProductName) -> Self {
        Self { id, name, domain_events: Vec::new() }
    }

    pub fn rename(&mut self, name: ProductName) {
        if self.name == name {
            return; // nothing changed, nothing happened
        }
        let previous = std::mem::replace(&mut self.name, name);
        self.record(ProductRenamedEvent::new(self.id.value(), previous.value(), self.name.value()));
    }

    /// Hands over the recorded events and clears them, so they are published once.
    pub fn pull_domain_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.domain_events)
    }

    fn record(&mut self, event: impl DomainEvent) {
        self.domain_events.push(Box::new(event));
    }

    pub fn id(&self) -> &ProductId {
        &self.id
    }

    pub fn name(&self) -> &ProductName {
        &self.name
    }
}
```

Two named constructors make rule 8 explicit: `create` means "this is happening now", while `reconstitute` means "this already existed".

---

## Publishing from the application service

```rust
// libs/catalog/src/product/application/create_product/product_creator.rs

pub struct ProductCreator {
    repository: Arc<dyn ProductRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl ProductCreator {
    pub fn new(repository: Arc<dyn ProductRepository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self { repository, event_bus }
    }

    pub async fn execute(&self, id: ProductId, name: ProductName) -> Result<(), ProductError> {
        let mut product = Product::create(id, name);

        self.repository.save(&product).await?; // persist first: on error, `?` returns and nothing is published
        self.event_bus.publish(product.pull_domain_events()).await; // publish second

        Ok(())
    }
}
```

A use case that modifies an existing aggregate follows the same shape: load, call the aggregate's behaviour, save, then pull and publish.

---

## Domain services never publish: the application wrapper does

The `EventBus` is an application-layer tool. It is injected only into application services. Domain code (aggregates and [domain services](hexagonal-architecture.md#when-a-use-case-needs-another-use-case-domain-services)) **records** events and never **publishes** them:

- **The domain never depends on a bus.** It decides what happened. When that gets announced is the application's concern.
- **One place publishes.** Every use case reads the same way: call the domain, then publish. No event is published twice or forgotten depending on which domain service a use case happens to call.
- **Only the application service knows when the use case has finished.** It publishes after its last write has succeeded.

The application calls the domain. When the domain finishes, the application publishes the events the domain recorded:

```
ProductRenamer (application)
├── ProductRenamer::rename (domain service)
│   ├── Product::rename             records ProductRenamedEvent
│   ├── ProductRepository::update
│   └── returns the Product         the events travel inside the aggregate
└── EventBus::publish(product.pull_domain_events())
```

```rust
// libs/catalog/src/product/domain/services/product_renamer.rs
// Domain service: records (through the aggregate), never publishes.

pub async fn rename(&self, id: &ProductId, name: ProductName) -> Result<Product, ProductError> {
    let mut product = self.finder.find(id).await?;

    product.rename(name); // records ProductRenamedEvent
    self.repository.update(&product).await?;

    Ok(product) // the recorded events go back to the caller inside the aggregate
}
```

```rust
// libs/catalog/src/product/application/rename_product/product_renamer.rs
// Application wrapper: calls the domain service, then publishes what it recorded.

pub async fn execute(&self, id: ProductId, name: ProductName) -> Result<(), ProductError> {
    let mut product = self.renamer.rename(&id, name).await?;

    self.event_bus.publish(product.pull_domain_events()).await;

    Ok(())
}
```

- **A domain service that changes aggregates returns them**, so their recorded events reach the application. A domain service that returns `()` after changing an aggregate loses its events.
- **When a handler or a subscriber needs a domain service, it gets its own application service.** That service is a wrapper with the same input: it calls the domain service and, when it finishes, publishes the events. Handlers and subscribers never call the domain service directly (see [CQRS](cqrs.md#handlers-never-call-domain-services) for the full example, with its handler).

---

## Subscribers

A subscriber reacts to one event type and delegates to an application service of its own module. It takes primitives from the event and builds its own module's Value Objects.

```rust
// libs/catalog/src/products_counter/application/increment_products_counter_on_product_created/
//     increment_products_counter_on_product_created.rs

pub struct IncrementProductsCounterOnProductCreated {
    incrementer: ProductsCounterIncrementer,
}

impl IncrementProductsCounterOnProductCreated {
    pub fn new(incrementer: ProductsCounterIncrementer) -> Self {
        Self { incrementer }
    }
}

#[async_trait]
impl DomainEventSubscriber<ProductCreatedEvent> for IncrementProductsCounterOnProductCreated {
    type Error = ProductsCounterError;

    async fn on(&self, event: &ProductCreatedEvent) -> Result<(), ProductsCounterError> {
        self.incrementer.execute(EventId::new(event.event_id())?).await
    }
}
```

The application service is where idempotency lives. Here the counter remembers which events it has already counted:

```rust
// libs/catalog/src/products_counter/application/increment_products_counter_on_product_created/
//     products_counter_incrementer.rs

pub struct ProductsCounterIncrementer {
    repository: Arc<dyn ProductsCounterRepository>,
}

impl ProductsCounterIncrementer {
    pub fn new(repository: Arc<dyn ProductsCounterRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, event_id: EventId) -> Result<(), ProductsCounterError> {
        let mut counter = self
            .repository
            .search()
            .await?
            .unwrap_or_else(ProductsCounter::initialize);

        if counter.has_counted(&event_id) {
            return Ok(()); // already processed: delivering it again changes nothing
        }

        counter.increment(event_id);
        self.repository.save(&counter).await
    }
}
```

---

## Wiring: the composition root

Subscribers are registered on the bus at startup, and the bus is injected into the application services that publish. When a subscriber's own service needs a bus (to publish further events or dispatch a command), create the buses first and register last. See [Dependency Injection](dependency-injection.md).

```rust
let counter_repository: Arc<dyn ProductsCounterRepository> =
    Arc::new(InMemoryProductsCounterRepository::new());

let event_bus = InMemoryEventBus::new();
event_bus.subscribe(IncrementProductsCounterOnProductCreated::new(
    ProductsCounterIncrementer::new(Arc::clone(&counter_repository)),
));
let event_bus: Arc<dyn EventBus> = Arc::new(event_bus);

let product_repository: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepository::new());
let creator = ProductCreator::new(Arc::clone(&product_repository), Arc::clone(&event_bus));
```

The event type is inferred from the subscriber's `DomainEventSubscriber<E>` impl, so `subscribe` needs no turbofish.

---

## Delivery guarantees

An in-memory bus gives **at-most-once** delivery. If the process dies between the save and the publish, or a subscriber fails, that reaction is lost (only logged). That is acceptable for best-effort side effects.

When a reaction must never be lost:

1. **Outbox.** Store the pulled events in an outbox table **in the same transaction** as the aggregate. A relay process reads the outbox and publishes to the broker, marking each event as sent.
2. **Broker with retries.** The broker redelivers until the subscriber acknowledges. The result is **at-least-once** delivery, which is why rule 11 (idempotent subscribers) is mandatory.

The `EventBus` trait does not change. Only the infrastructure implementation does.

---

## Testing

| What | How |
|---|---|
| Aggregate | `create` records the creation event. Each behaviour records its event. A no-op change records nothing. `pull_domain_events` empties the list. |
| Application service | An event bus mock records published event names. Assert the expected events are published after a successful save, and **nothing** is published when the save fails. |
| Subscriber + its application service | Feed an event built in the test. Assert the effect. Deliver the same event twice and assert the effect happens once. |
| Bus implementation | Tested once in the shared crate: routing by type, several subscribers per event, a failing subscriber does not stop the others or fail `publish`. |

```rust
pub struct EventBusMock {
    published: Mutex<Vec<&'static str>>,
}

impl EventBusMock {
    pub fn new() -> Self {
        Self { published: Mutex::new(Vec::new()) }
    }

    pub fn published_event_names(&self) -> Vec<&'static str> {
        self.published.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventBus for EventBusMock {
    async fn publish(&self, events: Vec<Box<dyn DomainEvent>>) {
        self.published
            .lock()
            .unwrap()
            .extend(events.iter().map(|event| event.event_name()));
    }
}
```

```rust
#[test]
fn it_records_a_created_event_when_created() {
    let mut product = Product::create(ProductIdMother::random(), ProductNameMother::random());

    let names: Vec<_> = product.pull_domain_events().iter().map(|e| e.event_name()).collect();

    assert_eq!(names, vec![ProductCreatedEvent::EVENT_NAME]);
}

#[test]
fn it_records_nothing_when_renamed_to_the_same_name() {
    let name = ProductNameMother::random();
    let mut product = Product::reconstitute(ProductIdMother::random(), name.clone());

    product.rename(name);

    assert!(product.pull_domain_events().is_empty());
}
```

---

## Anti-patterns

| Anti-pattern | Fix |
|---|---|
| Imperative or present-tense names (`CreateProductEvent`) | Past tense: `ProductCreatedEvent` |
| Publishing before persisting | Save first, then pull and publish |
| Building events in the application service | The aggregate records them inside its behaviour |
| Events carrying Value Objects or entities | Primitive payload; consumers build their own VOs |
| Recording events when loading from storage | Separate `create` (records) from `reconstitute` (records nothing) |
| Business logic inside a subscriber | Delegate to an application service in the consuming module |
| `EventBus` injected into a domain service, or a domain service publishing | The domain service returns the aggregate; the application service publishes its events |
| A handler or subscriber calling a domain service directly | An application wrapper calls the domain service, then publishes |
| Subscriber placed in the publishing module | It belongs to the module that reacts |
| Consumer importing the event from the publishing module | Move the event to the context's `shared` module (or `libs/shared` across contexts) |
| Non-idempotent subscriber | Deduplicate by `event_id` or make the effect naturally idempotent |
| Failing the command because a subscriber failed | The bus isolates subscriber failures; the fact already happened |
| Using events to enforce an invariant that must be atomic | Enforce it inside the aggregate |
| Calling `domain_event_type_id()` on a `Box`/`Arc` | Call it through `as_ref()` |

---

## Checklist

- [ ] Event named `NounPastEvent` with a constant `EVENT_NAME` = `<context>.<aggregate>.<past_verb>`
- [ ] Payload is primitives only, and self-contained
- [ ] Metadata (`event_id`, `aggregate_id`, `occurred_on`) embedded via `DomainEventMetadata`
- [ ] The aggregate records the event inside the behaviour that changes state; no-op changes record nothing
- [ ] `reconstitute` records nothing
- [ ] The application service saves first, then publishes `pull_domain_events()`
- [ ] Only application services receive the `EventBus`; domain services return the aggregates they changed
- [ ] The event type lives in the publishing module, the context's `shared` module, or `libs/shared`, depending on who uses it
- [ ] The subscriber lives in the consuming module, is named `ReactionOnNounPast`, and delegates to an application service
- [ ] The subscriber is idempotent
- [ ] The subscriber is registered on the bus in the composition root
- [ ] Tests cover recording, publish-after-save, nothing published on failure, and duplicate delivery
