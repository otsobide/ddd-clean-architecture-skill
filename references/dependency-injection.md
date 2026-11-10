# Dependency Injection

Every component receives its collaborators through its constructor, typed as **traits** (`Arc<dyn Trait>`), and never builds them itself. A single place per app, the **composition root**, creates the concrete implementations and wires everything together.

There is no DI container, no macros, no globals: just constructors and one function that calls them.

> **Golden rule: depend on traits, receive them through `new`, and build everything in one place, the composition root.**

---

## Table of contents

- [Why](#why)
- [Constructor injection in Rust](#constructor-injection-in-rust)
- [What gets injected where](#what-gets-injected-where)
- [Buses are dependencies too](#buses-are-dependencies-too)
- [EventBus: publishing facts](#eventbus-publishing-facts)
- [QueryBus: reading another module's data](#querybus-reading-another-modules-data)
- [CommandBus: making another module act](#commandbus-making-another-module-act)
- [Rules for injecting buses](#rules-for-injecting-buses)
- [The composition root](#the-composition-root)
- [Testing with injected dependencies](#testing-with-injected-dependencies)
- [Anti-patterns](#anti-patterns)
- [Checklist](#checklist)

---

## Why

- **Testability.** A use case that receives `Arc<dyn ProductRepository>` can be tested with a mock. One that builds its own Postgres repository cannot.
- **Swappable infrastructure.** In-memory in tests, Postgres in production, same use case code.
- **Explicit dependencies.** The constructor *is* the list of what a component needs. Nothing is hidden in globals.
- **One place to look.** To know how the app is assembled, read one function.
- **Checked by the compiler.** A missing or wrong dependency is a compile error, not a runtime surprise.

---

## Constructor injection in Rust

```rust
pub struct ProductCreator {
    repository: Arc<dyn ProductRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl ProductCreator {
    pub fn new(repository: Arc<dyn ProductRepository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self { repository, event_bus }
    }
}
```

- **`dyn Trait`**: the service depends on the port (a trait declared in `domain/`), never on an adapter such as `PostgresProductRepository`.
- **`Arc`**: one repository or bus instance is shared by many services.
- **`Send + Sync` supertraits** on every injectable trait (`pub trait ProductRepository: Send + Sync`), so the `Arc`s can cross threads in an async runtime.
- **Nothing is built inside.** A constructor stores what it receives. It never calls `InMemoryProductRepository::new()` or opens a connection.

### `Arc<dyn Trait>`, not generics

```rust
// ✗ Avoid: generic parameters for every dependency
pub struct ProductCreator<R: ProductRepository, B: EventBus> { /* ... */ }
```

Generic parameters spread to every type that holds the service (handlers, buses, app state), and make the composition root and the tests verbose. `Arc<dyn Trait>` keeps every type simple. The cost of dynamic dispatch is negligible next to the I/O these traits perform.

---

## What gets injected where

| Component | Receives | Never receives |
|---|---|---|
| Command application service | Its module's repository traits and domain services, `EventBus`; when needed `QueryBus`, `CommandBus`, and ports for external services | Another module's repositories or services; other application services |
| Query application service | Its module's repository traits and domain services; when needed `QueryBus` | `EventBus`, `CommandBus` (queries have no side effects) |
| Domain service | Its module's ports (`Arc<dyn Trait>`) and other domain services | Buses, application services |
| Command / query handler | Its application service, by value (one handler, one service) | Repositories or buses directly |
| Event subscriber | Its application service, by value | Repositories or buses directly |
| Repository adapter | Infrastructure resources: connection pool, client, configuration | Application services, buses |
| Entry point (controller, CLI, consumer) | `CommandBus`, `QueryBus` | Application services, repositories |

Handlers and subscribers hold no logic (see [CQRS](cqrs.md) and [Domain Events](domain-events.md)), so they only need the service they delegate to. Everything else is injected into the service.

---

## Buses are dependencies too

A use case often needs more than its own repository:

- **Publish what happened** → `EventBus`.
- **Read data owned by another module** → `QueryBus`.
- **Ask another module to change something** → `CommandBus`.

All three are traits from the shared CQRS and domain-events crates, so they are injected exactly like a repository: `Arc<dyn EventBus>`, `Arc<dyn QueryBus>`, `Arc<dyn CommandBus>`.

The buses are what keep modules decoupled. A module never imports another module's repository or application service ([Project Structure](project-structure.md)). It sends a message through a bus instead. The message types it sends are used by two modules (the sender and the handler), so they live in the context's `shared` module, together with the Value Objects they carry.

---

## EventBus: publishing facts

The most common injected bus. Command use cases save, then publish what their aggregate recorded:

```rust
pub async fn execute(&self, id: ProductId, name: ProductName) -> Result<(), ProductError> {
    let mut product = Product::create(id, name);

    self.repository.save(&product).await?;
    self.event_bus.publish(product.pull_domain_events()).await;

    Ok(())
}
```

Use it to let other modules **react** to a fact without the publisher knowing them. See [Domain Events](domain-events.md).

---

## QueryBus: reading another module's data

The `review` module must reject reviews of products that don't exist. Products belong to the `product` module, so `review` must not touch `ProductRepository`. It asks a query instead.

```
libs/catalog/src/
├── shared/                                       # used by product AND review
│   ├── domain/value_objects/product_id.rs
│   └── application/search_product/
│       ├── search_product_query.rs
│       └── search_product_response.rs
├── product/                                      # answers the query
│   └── application/search_product/
│       ├── search_product_query_handler.rs
│       └── product_searcher.rs
└── review/                                       # asks the query
    └── application/create_review/
        └── review_creator.rs
```

```rust
// libs/catalog/src/shared/application/search_product/search_product_query.rs
pub struct SearchProductQuery {
    pub id: ProductId,
}

impl Query for SearchProductQuery {}

// libs/catalog/src/shared/application/search_product/search_product_response.rs
pub struct SearchProductResponse {
    pub id: String,
    pub name: String,
}
```

`Search` queries answer with `Option<Response>` (`None` when nothing matches). `Find` queries fail with `NotFound`. Here, the asking module only has to check for `None`, without knowing the other module's error types.

```rust
// libs/catalog/src/review/application/create_review/review_creator.rs

pub struct ReviewCreator {
    repository: Arc<dyn ReviewRepository>,
    query_bus: Arc<dyn QueryBus>,
    event_bus: Arc<dyn EventBus>,
}

impl ReviewCreator {
    pub fn new(
        repository: Arc<dyn ReviewRepository>,
        query_bus: Arc<dyn QueryBus>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self { repository, query_bus, event_bus }
    }

    pub async fn execute(
        &self,
        id: ReviewId,
        product_id: ProductId,
        rating: ReviewRating,
    ) -> Result<(), ReviewError> {
        self.ensure_product_exists(&product_id).await?;

        let mut review = Review::create(id, product_id, rating);
        self.repository.save(&review).await?;
        self.event_bus.publish(review.pull_domain_events()).await;

        Ok(())
    }

    async fn ensure_product_exists(&self, product_id: &ProductId) -> Result<(), ReviewError> {
        let response = self
            .query_bus
            .ask(Box::new(SearchProductQuery { id: product_id.clone() }))
            .await
            .map_err(|error| ReviewError::Unexpected(error.to_string()))?;

        let product = response
            .downcast::<Option<SearchProductResponse>>()
            .expect("SearchProductQuery is answered with Option<SearchProductResponse>");

        match *product {
            Some(_) => Ok(()),
            None => Err(ReviewError::ProductNotFound(product_id.value().to_string())),
        }
    }
}
```

The `review` module depends only on `shared` (query, response, `ProductId`) and on the `QueryBus` trait. It does not know who answers the query or how products are stored.

---

## CommandBus: making another module act

Use an injected `CommandBus` when a use case **orchestrates** a process and decides that another module must do something now, for example importing a batch of products from an external source.

```
libs/catalog/src/
├── shared/
│   ├── domain/value_objects/{product_id.rs, product_name.rs}
│   └── application/create_product/create_product_command.rs   # dispatched by products_import, handled by product
├── product/
│   └── application/create_product/
│       ├── create_product_command_handler.rs
│       └── product_creator.rs
└── products_import/
    ├── domain/ports/products_source.rs                         # port: where the rows come from
    └── application/import_products/
        ├── import_products_command.rs
        ├── import_products_command_handler.rs
        └── products_importer.rs
```

```rust
// libs/catalog/src/products_import/application/import_products/products_importer.rs

pub struct ProductsImporter {
    source: Arc<dyn ProductsSource>,
    command_bus: Arc<dyn CommandBus>,
}

impl ProductsImporter {
    pub fn new(source: Arc<dyn ProductsSource>, command_bus: Arc<dyn CommandBus>) -> Self {
        Self { source, command_bus }
    }

    pub async fn execute(&self) -> Result<(), ProductsImportError> {
        for row in self.source.rows().await? {
            let command = CreateProductCommand {
                id: ProductId::new(row.id)?,
                name: ProductName::new(row.name)?,
            };

            self.command_bus
                .dispatch(Box::new(command))
                .await
                .map_err(|error| ProductsImportError::Unexpected(error.to_string()))?;
        }

        Ok(())
    }
}
```

The importer never touches `ProductRepository` or `ProductCreator`. Creating a product goes through the product module's own use case, with its rules and events.

**Events or commands?** Prefer events. Use an event when the current module only states a fact and doesn't care who reacts. Use a command only when the current use case owns the process and needs the other module to act now, and to know whether it failed.

---

## Rules for injecting buses

1. **Entry points get only `CommandBus` and `QueryBus`.** They never reach application services or repositories.
2. **Query use cases never get `EventBus` or `CommandBus`.** A query has no side effects. A query use case may get a `QueryBus` to read from other modules.
3. **A module never injects another module's repository or application service.** It goes through a bus.
4. **Prefer `EventBus` over `CommandBus`** for anything that is a reaction to a fact.
5. **Messages exchanged between modules live in `shared`**, together with the Value Objects they carry (or in `libs/shared` across bounded contexts).
6. **Inject only what is used.** If a service has a bus it never calls, remove it.

---

## The composition root

Each app has one composition root, in its `lib.rs` so that end-to-end tests can reuse it. It builds the whole object graph in a fixed order:

1. Infrastructure resources (configuration, connection pools, clients).
2. Adapters (repositories, external service clients).
3. **Buses, created empty and shared as `Arc` before anything else.**
4. Application services, each receiving only what it uses.
5. Handlers and subscribers, **registered on the buses last**.
6. What the entry points need (the command and query buses), handed back to them.

```rust
// apps/catalog_api/src/lib.rs

pub struct Buses {
    pub command_bus: Arc<dyn CommandBus>,
    pub query_bus: Arc<dyn QueryBus>,
}

pub fn compose() -> Buses {
    // 1-2. Adapters
    let product_repository: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepository::new());
    let review_repository: Arc<dyn ReviewRepository> = Arc::new(InMemoryReviewRepository::new());
    let products_source: Arc<dyn ProductsSource> = Arc::new(InMemoryProductsSource::new(Vec::new()));

    // 3. Buses: empty, shared before anything uses them
    let command_bus = Arc::new(InMemoryCommandBus::new());
    let query_bus = Arc::new(InMemoryQueryBus::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // 4. Application services
    let product_creator = ProductCreator::new(product_repository.clone(), event_bus.clone());
    let product_searcher = ProductSearcher::new(product_repository.clone());
    let review_creator =
        ReviewCreator::new(review_repository.clone(), query_bus.clone(), event_bus.clone());
    let products_importer = ProductsImporter::new(products_source.clone(), command_bus.clone());

    // 5. Handlers and subscribers, registered last
    command_bus
        .register(CreateProductCommandHandler::new(product_creator))
        .expect("CreateProductCommandHandler registered twice");
    command_bus
        .register(CreateReviewCommandHandler::new(review_creator))
        .expect("CreateReviewCommandHandler registered twice");
    command_bus
        .register(ImportProductsCommandHandler::new(products_importer))
        .expect("ImportProductsCommandHandler registered twice");
    query_bus
        .register(SearchProductQueryHandler::new(product_searcher))
        .expect("SearchProductQueryHandler registered twice");

    // 6. Entry points only get the command and query buses
    Buses { command_bus, query_bus }
}
```

`Arc<InMemoryCommandBus>` coerces to `Arc<dyn CommandBus>` automatically when passed to a constructor or stored in `Buses`.

### Why buses are created first and registered last

`ImportProductsCommandHandler` is registered on the command bus, but its `ProductsImporter` needs that same command bus to dispatch `CreateProductCommand`. The same happens with subscribers that publish further events, or query services that ask other queries.

This only works if the bus exists, and is shared, **before** the services that use it are built. That is why the in-memory buses register handlers through `&self` (with an internal `RwLock`) instead of `&mut self`:

```rust
impl InMemoryCommandBus {
    pub fn register<C, H>(&self, handler: H) -> Result<(), CommandBusError>
    where
        C: Command,
        H: CommandHandler<C> + 'static,
    { /* takes the write lock, inserts the handler */ }
}
```

When dispatching, the bus clones the handler out of the lock and releases the lock **before** awaiting it. This lets a handler dispatch another command on the same bus without deadlocking.

The resulting `Arc` cycle (bus → handler → service → bus) is intentional. Buses and services live for the whole process.

### Choosing adapters

The composition root is the **only** place that names concrete adapters, so it is also where configuration decides between them:

```rust
let product_repository: Arc<dyn ProductRepository> = match &config.database_url {
    Some(_) => Arc::new(PostgresProductRepository::new(pool.clone())),
    None => Arc::new(InMemoryProductRepository::new()),
};
```

---

## Testing with injected dependencies

Unit tests are their own small composition root: they build the service under test with mocks. Keep the concrete `Arc` of each mock to make assertions, and pass a clone, which coerces to `Arc<dyn Trait>`.

```rust
pub struct QueryBusMock {
    response: Mutex<Option<Box<dyn Any + Send + Sync>>>,
}

impl QueryBusMock {
    pub fn answering<R: Send + Sync + 'static>(response: R) -> Self {
        Self { response: Mutex::new(Some(Box::new(response))) }
    }
}

#[async_trait]
impl QueryBus for QueryBusMock {
    async fn ask(&self, _query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync>, QueryBusError> {
        Ok(self
            .response
            .lock()
            .unwrap()
            .take()
            .expect("QueryBusMock was asked more times than configured"))
    }
}
```

```rust
#[tokio::test]
async fn it_rejects_a_review_of_a_missing_product() {
    let repository = Arc::new(ReviewRepositoryMock::new());
    let query_bus = Arc::new(QueryBusMock::answering(None::<SearchProductResponse>));
    let event_bus = Arc::new(EventBusMock::new());
    let creator = ReviewCreator::new(repository.clone(), query_bus, event_bus.clone());

    let result = creator
        .execute(ReviewIdMother::random(), ProductIdMother::random(), ReviewRatingMother::random())
        .await;

    assert!(matches!(result, Err(ReviewError::ProductNotFound(_))));
    assert!(repository.saved().is_empty());
    assert!(event_bus.published_event_names().is_empty());
}
```

End-to-end tests call the app's real `compose()`, which wires in-memory adapters, and go through the entry points.

---

## Anti-patterns

| Anti-pattern | Fix |
|---|---|
| A service building its own dependencies (`InMemoryProductRepository::new()` inside `new`) | Receive `Arc<dyn Trait>` through the constructor |
| Global singletons (`static`, `OnceLock`, `lazy_static`) for repositories or buses | Build them in the composition root and inject them |
| Passing the whole app state or a "container" into services (service locator) | Inject only the dependencies the service uses |
| Depending on an adapter (`Arc<PostgresProductRepository>`) | Depend on the port (`Arc<dyn ProductRepository>`) |
| Generic type parameters for every dependency | `Arc<dyn Trait>` |
| Injecting another module's repository or application service | Ask a query / dispatch a command through the injected bus |
| A query use case with `EventBus` or `CommandBus` | Queries have no side effects; only repositories and `QueryBus` |
| Wiring spread across `main.rs`, controllers, and services | One composition root per app |
| Holding a bus lock while awaiting a handler | Clone the handler out of the lock, release it, then await |

---

## Checklist

- [ ] Every dependency is received through `new` as `Arc<dyn Trait>`, and every injectable trait is `Send + Sync`
- [ ] No constructor builds its own collaborators
- [ ] Each component gets only what it uses (see [What gets injected where](#what-gets-injected-where))
- [ ] Cross-module reads go through `QueryBus`, cross-module writes through events or `CommandBus`
- [ ] Messages exchanged between modules, and the VOs they carry, live in `shared`
- [ ] Query use cases have no `EventBus` or `CommandBus`
- [ ] One composition root per app, in `lib.rs`: adapters → buses → services → registration
- [ ] Buses register through `&self` and are shared before the services that use them are built
- [ ] Unit tests build the service with mocks; end-to-end tests reuse `compose()`
