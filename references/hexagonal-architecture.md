# Hexagonal Architecture

How each module is built. The domain sits at the center and declares what it needs from the outside as interfaces (**ports**). The outside implements them (**adapters**). Every source dependency points inward.

This reference covers abstractions, interfaces (introduced in Java, then in other languages, then Rust traits), the dependency inversion principle, ports and adapters, the three layers, use cases, and domain services.

> **Golden rule: dependencies point inward.** The domain depends on nothing outside itself. It declares the interfaces; infrastructure implements them.

---

## Table of contents

- [Why](#why)
- [Abstractions](#abstractions)
- [Interfaces](#interfaces)
  - [The concept, in Java](#the-concept-in-java)
  - [The same idea in other languages](#the-same-idea-in-other-languages)
  - [Rust: traits](#rust-traits)
- [Dependency inversion](#dependency-inversion)
- [Ports and adapters](#ports-and-adapters)
- [The three layers](#the-three-layers)
  - [Domain](#domain-the-model-and-its-contracts)
  - [Application](#application-the-use-cases)
  - [Infrastructure](#infrastructure-the-adapters)
- [Use cases](#use-cases)
- [When a use case needs another use case: domain services](#when-a-use-case-needs-another-use-case-domain-services)
- [A request through the hexagon](#a-request-through-the-hexagon)
- [Testing by layer](#testing-by-layer)
- [Anti-patterns](#anti-patterns)
- [Checklist](#checklist)

---

## Why

- **Business rules are testable on their own.** No database, no HTTP server, no network.
- **Technology changes stay local.** Moving from Postgres to another store, or from REST to gRPC, means writing a new adapter. Domain and application code don't change.
- **Infrastructure decisions can wait.** Start with in-memory adapters and plug in the real ones later.
- **The code reads like the business.** The domain contains only the ubiquitous language: `Product`, `rename`, `ProductRepository::search`.

---

## Abstractions

An abstraction states **what** something does and hides **how** it does it. The caller depends on the *what*.

"Store this product" is an abstraction. "Open a pooled connection and run `INSERT INTO products ...`" is a detail. A use case needs the first and should never see the second.

A good abstraction for a port:

- **Speaks the domain's language.** `ProductRepository::search(&ProductId)`, not `ProductsTable::select_by_pk(Uuid)`.
- **Uses domain types only.** It accepts and returns aggregates, Value Objects and domain errors, never `PgRow`, `sqlx::Error`, `reqwest::Response` or `serde_json::Value`.
- **Is as small as its consumers need.** A method no use case calls doesn't belong in it.
- **Doesn't leak its implementation.** No `begin_transaction()`, no SQL fragments, no pagination tokens of one specific API.

| Leaky | Abstract |
|---|---|
| `fn query(&self, sql: &str) -> Result<Vec<PgRow>, sqlx::Error>` | `fn search(&self, id: &ProductId) -> Result<Option<Product>, ProductError>` |
| `fn charge(&self, body: serde_json::Value) -> Result<reqwest::Response, reqwest::Error>` | `fn charge(&self, order_id: &OrderId, amount: &Money) -> Result<PaymentId, PaymentError>` |

In code, an abstraction is expressed as an **interface**.

---

## Interfaces

### The concept, in Java

Java has a keyword for it, so it is the clearest place to start. An interface is a contract: a list of method signatures without bodies. A class promises to fulfill it with `implements`, and the compiler checks the promise.

```java
// domain: the contract
public interface ProductRepository {
    void save(Product product);
    Optional<Product> search(ProductId id);
}
```

```java
// infrastructure: one implementation
public final class PostgresProductRepository implements ProductRepository {
    private final DataSource dataSource;

    public PostgresProductRepository(DataSource dataSource) {
        this.dataSource = dataSource;
    }

    @Override
    public void save(Product product) { /* INSERT ... */ }

    @Override
    public Optional<Product> search(ProductId id) { /* SELECT ... */ }
}
```

```java
// application: depends on the interface only
public final class ProductCreator {
    private final ProductRepository repository;

    public ProductCreator(ProductRepository repository) {
        this.repository = repository;
    }

    public void execute(ProductId id, ProductName name) {
        repository.save(Product.create(id, name));
    }
}
```

What matters here:

- `ProductCreator` names only `ProductRepository`. It compiles even if `PostgresProductRepository` doesn't exist.
- Any implementation can be passed in: Postgres in production, an in-memory one in tests.
- `repository.save(...)` is resolved at runtime against whatever object was passed (dynamic dispatch).

### The same idea in other languages

| Language | Construct | How a type fulfills it |
|---|---|---|
| Java, C#, Kotlin, PHP | `interface` | **Nominal**: the class declares it (`implements`, or `:` in C# and Kotlin) |
| TypeScript | `interface` | **Structural**: any object with the right shape fits, nothing to declare |
| Go | `interface` | **Structural**: a type that has the methods satisfies it implicitly |
| Python | `typing.Protocol` / `abc.ABC` | `Protocol` is structural (checked by type checkers). `ABC` is nominal (subclassing) |
| Rust | `trait` | **Nominal**: an explicit `impl Trait for Type` block |

Go shows the structural style well. The adapter never mentions the interface:

```go
type ProductRepository interface {
    Save(ctx context.Context, product Product) error
    Search(ctx context.Context, id ProductID) (*Product, error)
}

// PostgresProductRepository never names ProductRepository.
// Having both methods is enough to satisfy it.
type PostgresProductRepository struct{ db *sql.DB }
```

- **Nominal**: implementing is a deliberate declaration. Changing the interface breaks every implementor at compile time.
- **Structural**: a type can satisfy a contract it has never heard of. That is convenient, but a type can also fit by accident.

Both styles support hexagonal architecture. What matters is that the consumer depends on the contract, never on the implementation.

### Rust: traits

From here on, everything is Rust. A trait is Rust's interface: a set of method signatures that a type promises to implement.

```rust
// libs/catalog/src/product/domain/repositories/product_repository.rs

/// Port: the collection of `Product` aggregates.
/// Implemented in `infrastructure/persistence/`.
#[async_trait]
pub trait ProductRepository: Send + Sync {
    /// Stores a new product. Fails with `AlreadyExists` if the id is taken.
    async fn save(&self, product: &Product) -> Result<(), ProductError>;

    /// Replaces a stored product. Fails with `NotFound` if there is none.
    async fn update(&self, product: &Product) -> Result<(), ProductError>;

    /// Returns the product, or `None` if no product has this id.
    async fn search(&self, id: &ProductId) -> Result<Option<Product>, ProductError>;
}
```

```rust
// libs/catalog/src/product/domain/errors/product_error.rs

#[derive(Debug, Error)]
pub enum ProductError {
    #[error("product {0} not found")]
    NotFound(String),

    #[error("product {0} already exists")]
    AlreadyExists(String),

    #[error("unexpected error: {0}")]
    Unexpected(String),
}
```

An adapter implements it in its own file, in `infrastructure/`:

```rust
// libs/catalog/src/product/infrastructure/persistence/in_memory/in_memory_product_repository.rs

/// Stores the state, never the aggregate: a `Product` also carries its pending events.
#[derive(Default)]
pub struct InMemoryProductRepository {
    products: Mutex<HashMap<String, (ProductId, ProductName)>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn save(&self, product: &Product) -> Result<(), ProductError> {
        let mut products = self.products.lock().expect("products lock poisoned");
        let key = product.id().value().to_string();
        if products.contains_key(&key) {
            return Err(ProductError::AlreadyExists(key));
        }
        products.insert(key, (product.id().clone(), product.name().clone()));
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), ProductError> {
        let mut products = self.products.lock().expect("products lock poisoned");
        let key = product.id().value().to_string();
        if !products.contains_key(&key) {
            return Err(ProductError::NotFound(key));
        }
        products.insert(key, (product.id().clone(), product.name().clone()));
        Ok(())
    }

    async fn search(&self, id: &ProductId) -> Result<Option<Product>, ProductError> {
        let products = self.products.lock().expect("products lock poisoned");
        Ok(products
            .get(id.value())
            .map(|(id, name)| Product::reconstitute(id.clone(), name.clone())))
    }
}
```

#### From Java to Rust

| Java | Rust |
|---|---|
| `interface ProductRepository { ... }` | `trait ProductRepository { ... }` |
| `class PostgresProductRepository implements ProductRepository` | `impl ProductRepository for PostgresProductRepository`, a separate block |
| A field typed by the interface: `ProductRepository repository` | A trait object: `Arc<dyn ProductRepository>` (dynamic dispatch) |
| Bounded generic: `<R extends ProductRepository>` | Trait bound: `<R: ProductRepository>` or `impl ProductRepository` (static dispatch) |
| `interface A extends B` | Supertrait: `trait A: B`, as in `ProductRepository: Send + Sync` |
| `default` methods | Default methods in the trait body |
| Generic interface `CommandHandler<C, E>` | Generic trait with an associated type: `CommandHandler<C> { type Error; }` |
| Abstract class with fields | No equivalent: traits carry no data. Use composition |

Differences that matter for ports:

- **The implementation lives apart from the type.** The struct definition never mentions the trait. You can implement your own trait for any type, and any trait for your own type. You can't implement a foreign trait for a foreign type (the orphan rule).
- **Ports are used as `dyn Trait`.** Services hold `Arc<dyn ProductRepository>`, the Rust equivalent of a Java field typed by an interface. See [Dependency Injection](dependency-injection.md) for why `dyn` is preferred over generics.
- **`dyn` requires a dyn-compatible trait.** Its methods can't have generic type parameters or return `Self` by value. Native `async fn` in traits is not dyn compatible either, so async ports use `#[async_trait]`, which makes each method return a boxed future.
- **`Send + Sync` as supertraits.** These let an `Arc<dyn ProductRepository>` be shared between the async runtime's threads.

---

## Dependency inversion

The dependency inversion principle:

1. High-level modules should not depend on low-level modules. Both should depend on abstractions.
2. Abstractions should not depend on details. Details should depend on abstractions.

**High level** is the domain and the use cases: *what* the business does. **Low level** is Postgres, HTTP clients and queues: *how* it gets done.

### Without inversion

```rust
// ✗ The use case depends on infrastructure
use crate::product::infrastructure::persistence::postgres::PostgresProductRepository;

pub struct ProductCreator {
    repository: PostgresProductRepository,
}
```

```
ProductCreator ──► PostgresProductRepository ──► sqlx
 (application)        (infrastructure)
```

The use case can't be tested without a database, changing the database means changing the use case, and the business code drags `sqlx` along.

### With inversion

```rust
// ✓ The use case depends on a trait declared in the domain
pub struct ProductCreator {
    repository: Arc<dyn ProductRepository>,
}
```

```
Runtime calls:         ProductCreator ──► PostgresProductRepository ──► database

Source dependencies:   ProductCreator ──► ProductRepository ◄── PostgresProductRepository
                       (application)      (domain trait)        (infrastructure)
```

At runtime, the use case still ends up calling Postgres. In the source code, however, the arrow from infrastructure now points **toward the domain**: infrastructure depends on the domain, not the other way around. That reversal is the inversion.

### The port belongs to its consumer

- **The trait lives in the domain of the module that needs it**, never next to the adapter. If it lived in `infrastructure/`, the domain would have to import infrastructure to use it, and the arrow would point outward again.
- **The consumer shapes it.** Method names and types come from the domain, sized to what the use cases need.
- **The adapter adapts.** It translates domain types into rows, JSON or requests, and technology errors into domain errors. Nothing technological crosses the port.

```rust
// libs/catalog/src/product/infrastructure/persistence/postgres/postgres_product_repository.rs

pub struct PostgresProductRepository {
    pool: PgPool,
}

impl PostgresProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn save(&self, product: &Product) -> Result<(), ProductError> {
        sqlx::query("INSERT INTO products (id, name) VALUES ($1, $2)")
            .bind(product.id().value())
            .bind(product.name().value())
            .execute(&self.pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::Database(db_error) if db_error.is_unique_violation() => {
                    ProductError::AlreadyExists(product.id().value().to_string())
                }
                other => ProductError::Unexpected(other.to_string()),
            })?;
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), ProductError> {
        let result = sqlx::query("UPDATE products SET name = $2 WHERE id = $1")
            .bind(product.id().value())
            .bind(product.name().value())
            .execute(&self.pool)
            .await
            .map_err(|error| ProductError::Unexpected(error.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ProductError::NotFound(product.id().value().to_string()));
        }
        Ok(())
    }

    async fn search(&self, id: &ProductId) -> Result<Option<Product>, ProductError> {
        let row: Option<ProductRow> = sqlx::query_as("SELECT id, name FROM products WHERE id = $1")
            .bind(id.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| ProductError::Unexpected(error.to_string()))?;

        row.map(ProductRow::into_product).transpose()
    }
}

/// The table's shape. Private to the adapter: it never leaves this file.
#[derive(sqlx::FromRow)]
struct ProductRow {
    id: String,
    name: String,
}

impl ProductRow {
    /// Rebuilds the aggregate from storage with `reconstitute`, so no event is recorded.
    fn into_product(self) -> Result<Product, ProductError> {
        let id = ProductId::new(self.id).map_err(|error| ProductError::Unexpected(error.to_string()))?;
        let name = ProductName::new(self.name).map_err(|error| ProductError::Unexpected(error.to_string()))?;
        Ok(Product::reconstitute(id, name))
    }
}
```

`sqlx::Error` and `ProductRow` exist only inside this file. The rest of the system sees `Product` and `ProductError`.

---

## Ports and adapters

```
                        ┌──────────────────────────────────────┐
  HTTP controller ──┐   │ application                          │
  CLI command ──────┼──►│   use cases (CommandBus / QueryBus)  │
  queue consumer ───┘   │          │                           │
  driving adapters      │          ▼                           │
                        │ domain                               │
                        │   aggregates, VOs, events,           │   driven adapters
                        │   domain services                    │
                        │   ports: ProductRepository ◄─────────┼── PostgresProductRepository
                        │          EventBus ◄──────────────────┼── InMemoryEventBus
                        │          ProductsSource ◄────────────┼── CsvProductsSource
                        └──────────────────────────────────────┘

        ──►  calls                                    ◄──  implements
```

| Term | What it is | Examples | Where it lives |
|---|---|---|---|
| **Port** | A trait through which the hexagon talks to the outside | `ProductRepository`, `EventBus`, `ProductsSource` | `domain/` of the module, or a `libs/shared` crate's `domain/` for the buses |
| **Driven adapter** (secondary) | Implements a port; the application calls it | `PostgresProductRepository`, `InMemoryEventBus`, `CsvProductsSource` | `infrastructure/` of the module, or of the shared crate |
| **Driving adapter** (primary) | Calls the application from outside | actix controllers, CLI commands, queue consumers, scheduled jobs | The app (`apps/<app>/`) |

Driving adapters don't need one port per use case. `CommandBus` and `QueryBus` are the entry ports, and commands and queries are the messages that cross them (see [CQRS](cqrs.md)).

---

## The three layers

```
┌──────────────────────────────────────────────┐
│ infrastructure                               │
│   ┌──────────────────────────────────────┐   │
│   │ application                          │   │
│   │   ┌──────────────────────────────┐   │   │
│   │   │ domain                       │   │   │
│   │   └──────────────────────────────┘   │   │
│   └──────────────────────────────────────┘   │
└──────────────────────────────────────────────┘
          every source dependency points inward
```

Each module has the three layers ([Project Structure](project-structure.md)):

```
product/
├── domain/
│   ├── entities/            # Product (aggregate root)
│   ├── value_objects/       # ProductId, ProductName
│   ├── repositories/        # ProductRepository (trait only)
│   ├── ports/               # other driven ports, when the module has any
│   ├── events/              # ProductRenamedEvent
│   ├── errors/              # ProductError
│   └── services/            # domain services: ProductFinder
├── application/
│   ├── create_product/
│   ├── find_product/
│   └── rename_product/
└── infrastructure/
    └── persistence/
        ├── in_memory/       # InMemoryProductRepository
        └── postgres/        # PostgresProductRepository
```

### Domain: the model and its contracts

Everything that expresses the business, and nothing that performs I/O.

| Element | Folder | Example |
|---|---|---|
| Aggregate roots and entities | `domain/entities/` | `Product` |
| Value Objects | `domain/value_objects/` | `ProductId`, `ProductName` |
| Repository traits (the interface, never the implementation) | `domain/repositories/` | `ProductRepository` |
| Other driven ports | `domain/ports/` | `ProductsSource`, `PaymentGateway` |
| Domain events | `domain/events/` | `ProductRenamedEvent` |
| Domain errors | `domain/errors/` | `ProductError` |
| Domain services | `domain/services/` | `ProductFinder` |

- **Only interfaces for anything that touches the outside.** The repository *trait* is domain. Its implementations are infrastructure.
- **The repository trait belongs to the domain, not to application.** It describes the collection of aggregates in the domain's language, and domain services use it.
- **No frameworks and no I/O crates.** No actix-web, sqlx, reqwest or runtime APIs. Small utility crates (`thiserror`, `uuid`, `async-trait`) are fine.
- **Domain code never calls a bus** and never sees commands, queries or DTOs.
- **It may depend on:** its own domain, the context's `shared/domain`, and the `libs/shared` domain crates.

### Application: the use cases

What the system does, told as a sequence of steps. It has no business rules of its own.

- **Contains:** application services (one per use case), commands, queries and their handlers, response DTOs, and event subscribers.
- **Orchestrates:** load through a port, call domain behaviour, persist, publish events.
- **Delegates decisions to the domain.** The aggregate (`Product::rename` ignores a rename to the same name) or a domain service decides. The application service never does.
- **Translates outward:** handlers map aggregates to response DTOs.
- **It may depend on:** the domain. Never on infrastructure, and never on another use case (see [domain services](#when-a-use-case-needs-another-use-case-domain-services)).

### Infrastructure: the adapters

How things get done with real technology.

- **Contains:** repository implementations (in-memory, Postgres), clients for external APIs, and mappers between storage and aggregates. The bus implementations live in the shared crates' `infrastructure/`.
- **Translates both ways:** domain types to rows or requests and back, and technology errors to domain errors.
- **No business rules.** No "only update if the name changed" in SQL; that decision belongs to the aggregate.
- **Nobody calls it directly.** Application and domain code reach it only through the trait. The concrete adapter is chosen in the app's composition root ([Dependency Injection](dependency-injection.md)).
- **It may depend on:** domain, application, and external crates.

---

## Use cases

A use case is one thing a user or another system wants to do: create a product, rename it, find it. In code, it is a folder in `application/` with the message, its handler, and one **application service**:

```
product/application/rename_product/
├── rename_product_command.rs
├── rename_product_command_handler.rs
└── product_renamer.rs
```

An application service:

- Is named `<Noun><Verb>er`: `ProductCreator`, `ProductRenamer`, `ProductFinder`.
- Has one public method, `execute`, which takes Value Objects.
- Receives its dependencies through the constructor: ports, domain services, buses.
- Returns `Result<(), DomainError>` for a command, or the aggregate for a query (the handler maps it to a DTO, see [CQRS](cqrs.md)).

A command use case always follows the same four steps:

```rust
// libs/catalog/src/product/application/rename_product/product_renamer.rs

pub struct ProductRenamer {
    repository: Arc<dyn ProductRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl ProductRenamer {
    pub fn new(repository: Arc<dyn ProductRepository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self { repository, event_bus }
    }

    pub async fn execute(&self, id: ProductId, name: ProductName) -> Result<(), ProductError> {
        // 1. Load the aggregate through the port.
        let mut product = self
            .repository
            .search(&id)
            .await?
            .ok_or_else(|| ProductError::NotFound(id.value().to_string()))?;

        // 2. Let the aggregate apply the change: the rule lives there.
        product.rename(name);

        // 3. Persist through the port.
        self.repository.update(&product).await?;

        // 4. Publish what the aggregate recorded.
        self.event_bus.publish(product.pull_domain_events()).await;

        Ok(())
    }
}
```

What does **not** belong in an application service:

| Doesn't belong | Where it goes |
|---|---|
| Business rules (`if product.name() == &name`, length limits) | The aggregate or a Value Object |
| SQL, HTTP calls, serialization | An adapter behind a port |
| Mapping to response DTOs | The query handler |
| Calling another use case | A domain service (next section) |

---

## When a use case needs another use case: domain services

`ProductRenamer` above loads the product or fails with `NotFound`. The find use case already does exactly that ([CQRS](cqrs.md)):

```rust
// libs/catalog/src/product/application/find_product/product_finder.rs
self.repository
    .search(&id)
    .await?
    .ok_or_else(|| ProductError::NotFound(id.value().to_string()))
```

`ProductDeleter` will need the same thing. There are two tempting answers, and both are wrong.

**✗ Inject one application service into another.**

```rust
use crate::product::application::find_product::product_finder::ProductFinder;

pub struct ProductRenamer {
    finder: ProductFinder, // ✗ a use case inside a use case
    // ...
}
```

- **A use case is an entry point.** Each represents one intention and is reached through a bus by a driving adapter. Nesting them hides a second intention inside the first.
- **Use cases become coupled.** Adding a cache, authorization or a different response to the find use case silently changes rename.
- **Side effects leak.** Whatever the reused use case does (publishing events, logging, checking permissions), the caller inherits.
- **Application starts depending on application.** The dependency graph grows sideways instead of inward.

**✗ Copy the lines into every use case.** The rule (how to load, which error) ends up in several places that drift apart.

**✓ Move it to the domain.** Logic needed by more than one use case is domain logic. It becomes a **domain service**: a use case modularized into a reusable domain piece. The application services that need it use the domain service, and none of them knows about the others.

```
Before                                        After

find_product/ProductFinder   ◄── ✗ ──┐        find_product/ProductFinder   ──┐
rename_product/ProductRenamer ───────┘        rename_product/ProductRenamer ─┼──► domain/services/ProductFinder
                                              delete_product/ProductDeleter ─┘            │
                                                                                          ▼
                                                                                  ProductRepository
```

```rust
// libs/catalog/src/product/domain/services/product_finder.rs

/// Domain service: finds a product or fails with `ProductError::NotFound`.
pub struct ProductFinder {
    repository: Arc<dyn ProductRepository>,
}

impl ProductFinder {
    pub fn new(repository: Arc<dyn ProductRepository>) -> Self {
        Self { repository }
    }

    pub async fn find(&self, id: &ProductId) -> Result<Product, ProductError> {
        self.repository
            .search(id)
            .await?
            .ok_or_else(|| ProductError::NotFound(id.value().to_string()))
    }
}
```

The rename use case uses it to load the product:

```rust
// libs/catalog/src/product/application/rename_product/product_renamer.rs

use crate::product::domain::services::product_finder::ProductFinder;

pub struct ProductRenamer {
    finder: ProductFinder,                  // domain service
    repository: Arc<dyn ProductRepository>, // port
    event_bus: Arc<dyn EventBus>,           // port
}

impl ProductRenamer {
    pub fn new(
        finder: ProductFinder,
        repository: Arc<dyn ProductRepository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self { finder, repository, event_bus }
    }

    pub async fn execute(&self, id: ProductId, name: ProductName) -> Result<(), ProductError> {
        let mut product = self.finder.find(&id).await?;

        product.rename(name);

        self.repository.update(&product).await?;
        self.event_bus.publish(product.pull_domain_events()).await;

        Ok(())
    }
}
```

The find use case keeps its own application service, which is now a thin entry point:

```rust
// libs/catalog/src/product/application/find_product/product_finder.rs

use crate::product::domain::services::product_finder::ProductFinder as DomainProductFinder;

pub struct ProductFinder {
    finder: DomainProductFinder,
}

impl ProductFinder {
    pub fn new(finder: DomainProductFinder) -> Self {
        Self { finder }
    }

    pub async fn execute(&self, id: ProductId) -> Result<Product, ProductError> {
        self.finder.find(&id).await
    }
}
```

Both services are named after what they do, and their paths tell them apart. Import the domain one as `DomainProductFinder` wherever both would be in scope.

Domain services are concrete structs, not traits. They are domain logic with a single implementation, not a boundary to the outside. They are built in the composition root and passed by value:

```rust
let product_repository: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepository::new());

let renamer = ProductRenamer::new(
    DomainProductFinder::new(product_repository.clone()),
    product_repository.clone(),
    event_bus.clone(),
);
let finder = ProductFinder::new(DomainProductFinder::new(product_repository.clone()));
```

**When to extract.** As soon as a second use case needs logic that an application service already has, that logic moves to `domain/services/`. Not before. This is the same rule as moving things to `shared`.

Typical domain services:

- **Find or fail:** `ProductFinder` (above).
- **A check shared by several use cases:** for example, "no other product has this name", needed by both create and rename.
- **A calculation spanning several aggregates of the module**, which doesn't belong to any one of them.

### Rules for domain services

1. **They live in `domain/services/` of the module that owns the aggregate.**
2. **They depend only on the domain:** ports, aggregates, Value Objects, and other domain services.
3. **They never receive a bus and never publish events.** Aggregates record events, and the application service publishes them.
4. **They never take or return commands, queries or DTOs.** They speak in aggregates, Value Objects and domain errors.
5. **They are stateless:** they hold dependencies, never data between calls.
6. **Their method is named after the operation** (`find`, `ensure_name_is_free`), not `execute`. They are building blocks, not entry points.
7. **They never cross module boundaries.** Another module that needs the same answer asks a query through the `QueryBus` ([Dependency Injection](dependency-injection.md)).
8. **Handlers and subscribers never call them.** A handler or subscriber that needs a domain service gets an application service wrapping it: same input, calls the domain service, then publishes the recorded events ([CQRS](cqrs.md#handlers-never-call-domain-services), [Domain Events](domain-events.md#domain-services-never-publish-the-application-wrapper-does)).

---

## A request through the hexagon

```
PUT /products/{id}                                 driving adapter (apps/catalog_api)
└── controller builds RenameProductCommand         (VO validation fails → 400)
    └── CommandBus::dispatch                       entry port (libs/shared)
        └── RenameProductCommandHandler            application
            └── ProductRenamer::execute            application
                ├── ProductFinder::find            domain service
                │   └── ProductRepository::search  port ──► PostgresProductRepository
                ├── Product::rename                aggregate: records ProductRenamedEvent
                ├── ProductRepository::update      port ──► PostgresProductRepository
                └── EventBus::publish              port ──► InMemoryEventBus
```

Every layer change in this trace goes through a trait. Swap any adapter on the right and nothing above it changes.

---

## Testing by layer

| Layer | What is tested | Test doubles |
|---|---|---|
| Domain | Aggregates, Value Objects | None: pure code |
| Domain | Domain services | Mocks of the ports they receive |
| Application | Application services | Mocks of the ports (repository, `EventBus`, `QueryBus`) |
| Infrastructure | Adapters | None: run them against the real technology (integration tests) |
| App | The whole request | In-memory adapters, real HTTP stack (end-to-end tests) |

Because use cases depend only on traits, every application test runs in memory and in milliseconds.

---

## Anti-patterns

| Anti-pattern | Fix |
|---|---|
| Repository trait in `infrastructure/`, next to its implementation | Declare it in `domain/repositories/`; infrastructure implements it |
| A port that returns technology types (`sqlx::Error`, `PgRow`, `reqwest::Response`) | Return domain types and errors; the adapter translates |
| A port that mirrors a database API (`begin`, `commit`, `execute_sql`) | Shape it after what the use cases need |
| Application service holding a concrete adapter (`PostgresProductRepository`) | Depend on `Arc<dyn ProductRepository>` |
| Application service injecting another application service | Extract the shared logic into a domain service |
| The same lookup or check copied into several use cases | One domain service in `domain/services/` |
| Business rules in the application service | Move them into the aggregate or a Value Object |
| Domain service receiving a bus or publishing events | The aggregate records, the application service publishes |
| A handler or subscriber calling a domain service | Wrap it in an application service that publishes when it finishes |
| Domain importing actix-web, sqlx or reqwest | Keep frameworks in infrastructure and apps |
| Domain service hidden behind a trait "just in case" | Traits are for boundaries with several implementations; domain services are concrete |
| A module using another module's domain service | Ask a query through the `QueryBus` |

---

## Checklist

- [ ] Every trait the domain needs from the outside is declared in `domain/` (`repositories/` or `ports/`)
- [ ] Ports use domain names and types only; no technology type crosses them
- [ ] Adapters live in `infrastructure/`, translate errors, and rebuild aggregates with `reconstitute`
- [ ] Application services depend on `Arc<dyn Trait>`, never on an adapter
- [ ] Entities, Value Objects, repository traits, events, errors and domain services are in `domain/`
- [ ] The domain imports no framework or I/O crate
- [ ] Each use case has one application service with one `execute` method
- [ ] Application services orchestrate; decisions live in aggregates, Value Objects or domain services
- [ ] No application service uses another application service
- [ ] Logic needed by two use cases is a domain service in `domain/services/`
- [ ] Domain services never receive buses, publish events, or handle DTOs
