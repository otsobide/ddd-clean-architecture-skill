# Testing

How to test a hexagonal module. The ports let a test replace the whole infrastructure layer with hand-written **mocks**, while the domain and application code under test stay real. **Object Mothers** build the test data: one mother per Value Object and per aggregate, on top of a few primitive mothers that generate random strings and numbers.

> **Golden rule: mock only the ports.** Everything inside the hexagon (aggregates, Value Objects, domain services, application services) is the real code. Every value the test doesn't care about comes from a mother, at random. Every value it does care about is written in the test.

---

## Table of contents

- [Why the architecture makes testing easy](#why-the-architecture-makes-testing-easy)
- [What to test in each layer](#what-to-test-in-each-layer)
- [Layout](#layout)
- [Mocks](#mocks)
- [Object Mothers](#object-mothers)
  - [Primitive mothers](#primitive-mothers)
  - [Value Object mothers](#value-object-mothers)
  - [Aggregate mothers](#aggregate-mothers)
- [Unit tests of application services](#unit-tests-of-application-services)
- [Unit tests of the domain](#unit-tests-of-the-domain)
- [Adapter tests: one contract, every adapter](#adapter-tests-one-contract-every-adapter)
- [End-to-end tests](#end-to-end-tests)
- [Anti-patterns](#anti-patterns)
- [Checklist](#checklist)

---

## Why the architecture makes testing easy

An application service only knows traits ([Hexagonal Architecture](hexagonal-architecture.md)). Production plugs real adapters into them. A unit test plugs mocks into the same traits:

```
Production                                          Unit test

ProductRenamer                                      ProductRenamer
├── ProductFinder (domain service)                  ├── ProductFinder (domain service, real)
│   └── Arc<dyn ProductRepository>                  │   └── ProductRepositoryMock
│       = PostgresProductRepository                 │
├── Arc<dyn ProductRepository>                      ├── ProductRepositoryMock
│   = PostgresProductRepository                     │
└── Arc<dyn EventBus> = RabbitMqEventBus            └── EventBusMock

          domain + application: the same code; only the infrastructure layer is swapped
```

- **No test code in production.** No `#[cfg(test)]` branches, no feature flags, no "test mode".
- **The test is a small composition root.** It builds the service under test with mocks instead of adapters ([Dependency Injection](dependency-injection.md)).
- **Unit tests run in memory and in milliseconds.** They need no database, no network and no Docker.

There are three ways to put something behind a port:

| Replacement | What it is | Used in |
|---|---|---|
| **Mock** (hand-written) | Implements the port. Returns what the test configured and records what it received | Unit tests of application and domain services |
| **In-memory adapter** | A real, working implementation without I/O (`InMemoryProductRepository`) | End-to-end tests, local development |
| **Real adapter** | The production implementation (`PostgresProductRepository`) | Integration tests of that adapter |

---

## What to test in each layer

| Layer | Subject | Real | Replaced | Kind |
|---|---|---|---|---|
| Domain | Value Objects | Everything | Nothing | Unit |
| Domain | Aggregates: behaviour and recorded events | Everything | Nothing | Unit |
| Domain | Domain services | The service, aggregates, VOs | Ports → mocks | Unit |
| Application | Application services (commands, queries, subscribers) | Application and domain code | Ports → mocks | Unit |
| Infrastructure | Adapters | The adapter and its technology | Nothing | Integration |
| App | The whole request through the entry point | Everything | Adapters → in-memory adapters | End-to-end |

---

## Layout

Tests live outside the crates and mirror the source tree ([Project Structure](project-structure.md)). Primitive mothers and the mocks of shared ports are used by every context, so they live in a shared test-support crate:

```
libs/shared/testing/                    # crate shared-testing, only ever a dev-dependency
└── src/
    ├── mothers/                        # UuidMother, WordMother, IntegerMother
    └── mocks/                          # EventBusMock, QueryBusMock, CommandBusMock

tests/libs/catalog/
├── tests.rs
└── src/
    ├── mocks/                          # mocks of this context's ports
    │   └── product_repository_mock.rs
    └── product/                        # mirrors libs/catalog/src/product
        ├── domain/
        │   ├── value_objects/
        │   │   ├── mothers/            # one mother per VO
        │   │   │   ├── product_id_mother.rs
        │   │   │   └── product_name_mother.rs
        │   │   └── product_name_tests.rs
        │   ├── entities/
        │   │   ├── mothers/product_mother.rs
        │   │   └── product_tests.rs
        │   └── services/product_finder_tests.rs
        ├── application/
        │   └── rename_product/product_renamer_tests.rs
        └── infrastructure/
            └── persistence/
                ├── product_repository_contract.rs
                ├── in_memory_product_repository_tests.rs
                └── postgres_product_repository_tests.rs

tests/apps/catalog_api/                 # end-to-end tests
```

```toml
# libs/catalog/Cargo.toml
[dev-dependencies]
shared-testing = { path = "../shared/testing" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

A mother mirrors the path of the type it builds. If the type moves to a `shared` module, its mother moves with it.

---

## Mocks

A mock is a hand-written struct in the test crate that implements a port trait. It has two jobs:

- **Stub:** return what the test configured, such as "`search` finds this product" or "`update` fails".
- **Spy:** record what it received, so the test can assert it, such as "`update` received a product named X".

Write mocks by hand rather than generating them. A mock implements the same trait as the adapters, so the code under test can't tell the difference. When the port changes, every mock stops compiling. There are no macros and no expectation DSL, so a reader understands the mock by reading it.

```rust
// tests/libs/catalog/src/mocks/product_repository_mock.rs

/// Hand-written mock of the `ProductRepository` port.
/// Stub: what `search` finds and which writes fail. Spy: what was saved and updated.
#[derive(Default)]
pub struct ProductRepositoryMock {
    stored: Vec<Product>,
    save_fails: bool,
    update_fails: bool,
    saved: Mutex<Vec<Product>>,
    updated: Mutex<Vec<Product>>,
}

impl ProductRepositoryMock {
    /// Finds nothing, and every write succeeds.
    pub fn new() -> Self {
        Self::default()
    }

    /// `search` finds a copy of this product, by its id.
    pub fn finding(mut self, product: &Product) -> Self {
        self.stored.push(copy_of(product));
        self
    }

    /// `save` fails with `AlreadyExists`.
    pub fn failing_on_save(mut self) -> Self {
        self.save_fails = true;
        self
    }

    /// `update` fails with `Unexpected`.
    pub fn failing_on_update(mut self) -> Self {
        self.update_fails = true;
        self
    }

    pub fn saved(&self) -> Vec<Product> {
        self.saved.lock().unwrap().iter().map(copy_of).collect()
    }

    pub fn updated(&self) -> Vec<Product> {
        self.updated.lock().unwrap().iter().map(copy_of).collect()
    }
}

#[async_trait]
impl ProductRepository for ProductRepositoryMock {
    async fn save(&self, product: &Product) -> Result<(), ProductError> {
        if self.save_fails {
            return Err(ProductError::AlreadyExists(product.id().value().to_string()));
        }
        self.saved.lock().unwrap().push(copy_of(product));
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), ProductError> {
        if self.update_fails {
            return Err(ProductError::Unexpected("update failed (mock)".to_string()));
        }
        self.updated.lock().unwrap().push(copy_of(product));
        Ok(())
    }

    async fn search(&self, id: &ProductId) -> Result<Option<Product>, ProductError> {
        Ok(self.stored.iter().find(|product| product.id() == id).map(copy_of))
    }
}

/// `Product` isn't `Clone` (it owns its pending events), so the mock keeps
/// copies of its state, as a real repository would.
fn copy_of(product: &Product) -> Product {
    Product::reconstitute(product.id().clone(), product.name().clone())
}
```

Mock conventions:

- **Named `<Port>Mock`.** Mocks of a context's ports live in `tests/libs/<context>/src/mocks/`. Mocks of shared ports (`EventBusMock`, `QueryBusMock`, `CommandBusMock`) live in `libs/shared/testing`. See [Domain Events](domain-events.md#testing) and [Dependency Injection](dependency-injection.md#testing-with-injected-dependencies) for those two.
- **`new()` is the happy default.** It finds nothing, and every write succeeds.
- **Configuration methods take and return `self`** (`finding`, `failing_on_update`), so the setup reads as one line: `Arc::new(ProductRepositoryMock::new().finding(&product))`.
- **Spy accessors return what was received** (`saved()`, `updated()`, `published_event_names()`), so tests assert values, not just call counts.
- **Match on the arguments.** `search` finds by id. A mock that returns the product whatever id it is asked for can't catch a use case that loads the wrong product.
- **No logic beyond that.** A mock that starts behaving like a database has become an in-memory adapter. Use the adapter instead.

---

## Object Mothers

An Object Mother is a factory of **valid** test objects. By default every value is random. A test fixes only the values it is about.

```
primitive mothers              Value Object mothers            aggregate mothers
(libs/shared/testing)          (one per VO)                    (one per aggregate)

UuidMother     ─────────────►  ProductIdMother     ─────────►  ProductMother
WordMother     ─────────────►  ProductNameMother   ─────────►
IntegerMother  ─────────────►  UserAgeMother       ─────────►  UserMother
```

Why:

- **Intent.** In a test built with mothers, every literal matters, because everything else comes from `random()`. `ProductMother::with_name(ProductNameMother::create("Chair"))` says that this test is about the name.
- **No construction noise.** One call instead of three constructors and their `unwrap`s.
- **No accidental coupling.** Tests can't depend on a value they never chose, such as two tests sharing the id `"p-1"`. Random ids also let integration tests share a database without cleanup.
- **One place to change.** When a constructor changes, only its mother changes.

Random values are always valid, so a test fails at random only when the code under test has a bug for some valid input. That is a real bug. VOs derive `Debug`, so `assert_eq!` prints the values that failed.

### Primitive mothers

They generate random primitives. They know nothing about the domain:

```rust
// libs/shared/testing/src/mothers/uuid_mother.rs

pub struct UuidMother;

impl UuidMother {
    pub fn random() -> String {
        Uuid::new_v4().to_string()
    }
}
```

```rust
// libs/shared/testing/src/mothers/integer_mother.rs

use rand::distr::uniform::SampleUniform;
use rand::RngExt;

pub struct IntegerMother;

impl IntegerMother {
    /// A random integer in `min..=max`, for any integer type.
    pub fn between<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
        rand::rng().random_range(min..=max)
    }
}
```

```rust
// libs/shared/testing/src/mothers/word_mother.rs

use rand::RngExt;

pub struct WordMother;

impl WordMother {
    /// A random lowercase word of 3 to 12 letters.
    pub fn random() -> String {
        Self::with_length(IntegerMother::between(3, 12))
    }

    /// A random lowercase word of exactly `length` letters.
    pub fn with_length(length: usize) -> String {
        let mut rng = rand::rng();
        (0..length).map(|_| rng.random_range(b'a'..=b'z') as char).collect()
    }
}
```

Add primitive mothers when a VO needs them: `EmailMother`, `SentenceMother`, `DateMother`, and so on. If you'd rather not write them, a fake-data crate (such as `fake`) can generate the values inside these mothers. Tests still only call the mothers.

### Value Object mothers

There is **one mother per VO**. Every VO mother has:

- **`create(value)`**: builds the VO from a concrete primitive. It is the only place in the tests where a primitive becomes this VO.
- **`random()`**: a valid random value, built from primitive mothers.
- **Named constructors with domain meaning** when tests need a particular kind of value: `adult()`, `minor()`, `longest()`, `empty()` for nullable VOs ([Value Objects](value-objects.md#object-mothers)).

```rust
// tests/libs/catalog/src/product/domain/value_objects/mothers/product_id_mother.rs

pub struct ProductIdMother;

impl ProductIdMother {
    pub fn create(value: impl Into<String>) -> ProductId {
        ProductId::new(value).expect("ProductIdMother: invalid product id")
    }

    pub fn random() -> ProductId {
        Self::create(UuidMother::random())
    }
}
```

```rust
// tests/libs/catalog/src/product/domain/value_objects/mothers/product_name_mother.rs

pub struct ProductNameMother;

impl ProductNameMother {
    pub fn create(value: impl Into<String>) -> ProductName {
        ProductName::new(value).expect("ProductNameMother: invalid product name")
    }

    pub fn random() -> ProductName {
        Self::create(WordMother::random())
    }

    /// The longest valid name: for tests about the length limit.
    pub fn longest() -> ProductName {
        Self::create(WordMother::with_length(100))
    }
}
```

```rust
// tests/libs/accounts/src/user/domain/value_objects/mothers/user_age_mother.rs

pub struct UserAgeMother;

impl UserAgeMother {
    pub fn create(value: u8) -> UserAge {
        UserAge::new(value).expect("UserAgeMother: invalid age")
    }

    pub fn random() -> UserAge {
        Self::create(IntegerMother::between(0, UserAge::MAX))
    }

    pub fn adult() -> UserAge {
        Self::create(IntegerMother::between(UserAge::ADULTHOOD, UserAge::MAX))
    }

    pub fn minor() -> UserAge {
        Self::create(IntegerMother::between(0, UserAge::ADULTHOOD - 1))
    }
}
```

A mother calls `expect`, never returns `Result`. If a test passes an invalid value to a mother, the test itself is wrong. To test **invalid** input, skip the mother and call the VO's constructor with a primitive. An invalid VO can't exist, so no mother can return one:

```rust
#[test]
fn it_rejects_a_name_longer_than_100_characters() {
    assert!(ProductName::new(WordMother::with_length(101)).is_err());
}
```

### Aggregate mothers

There is **one mother per aggregate**. It builds the aggregate from VO mothers:

- **`create(...)`**: every value given, as VOs.
- **`random()`**: every value random.
- **`with_<field>(vo)`**: everything random **except** that field. Add one only when a test needs it.

```rust
// tests/libs/catalog/src/product/domain/entities/mothers/product_mother.rs

pub struct ProductMother;

impl ProductMother {
    /// Every value given.
    pub fn create(id: ProductId, name: ProductName) -> Product {
        Product::reconstitute(id, name)
    }

    /// Every value random.
    pub fn random() -> Product {
        Self::create(ProductIdMother::random(), ProductNameMother::random())
    }

    /// Random, except the id.
    pub fn with_id(id: ProductId) -> Product {
        Self::create(id, ProductNameMother::random())
    }

    /// Random, except the name.
    pub fn with_name(name: ProductName) -> Product {
        Self::create(ProductIdMother::random(), name)
    }
}
```

The same pattern works for an aggregate with more fields. A test about age fixes the age and leaves the rest random:

```rust
// tests/libs/accounts/src/user/domain/entities/mothers/user_mother.rs

pub struct UserMother;

impl UserMother {
    pub fn create(id: UserId, name: UserName, age: UserAge) -> User {
        User::reconstitute(id, name, age)
    }

    pub fn random() -> User {
        Self::create(UserIdMother::random(), UserNameMother::random(), UserAgeMother::random())
    }

    /// Random, except the age.
    pub fn with_age(age: UserAge) -> User {
        Self::create(UserIdMother::random(), UserNameMother::random(), age)
    }
}
```

```rust
#[test]
fn a_minor_is_not_an_adult() {
    let user = UserMother::with_age(UserAgeMother::minor());

    assert!(!user.is_adult());
}

#[test]
fn a_user_is_an_adult_from_18() {
    let user = UserMother::with_age(UserAgeMother::create(18));

    assert!(user.is_adult());
}
```

The first test only cares that the user is a minor, so the age itself is random. The second is about the boundary, so it writes `18`.

Rules for aggregate mothers:

- **Build with `reconstitute`, not `create`.** A mother returns an existing aggregate with no pending events, so no stray event leaks into the test. A test about creation calls `Product::create` itself.
- **`with_<field>` takes the VO, not a primitive:** `with_name(ProductNameMother::create("Chair"))`. Turning a primitive into a VO happens only in the VO's mother.
- **When tests need combinations** (age and name, age and email), don't multiply `with_age_and_name` methods. Turn that mother into a builder: `UserMother::builder().age(..).name(..).build()`, with every field random unless set.

Commands, queries and events can have mothers too, built the same way from VO mothers.

---

## Unit tests of application services

Each test builds the service under test as a small composition root. It keeps the concrete `Arc` of each mock for its assertions and passes clones, which coerce to `Arc<dyn Trait>`. The domain service is real: only the port beneath it is mocked.

```rust
// tests/libs/catalog/src/product/application/rename_product/product_renamer_tests.rs

fn renamer(repository: &Arc<ProductRepositoryMock>, event_bus: &Arc<EventBusMock>) -> ProductRenamer {
    ProductRenamer::new(
        ProductFinder::new(repository.clone()), // the real domain service, on the mock
        repository.clone(),
        event_bus.clone(),
    )
}

#[tokio::test]
async fn it_renames_an_existing_product() {
    let product = ProductMother::random();
    let new_name = ProductNameMother::random();
    let repository = Arc::new(ProductRepositoryMock::new().finding(&product));
    let event_bus = Arc::new(EventBusMock::new());

    renamer(&repository, &event_bus)
        .execute(product.id().clone(), new_name.clone())
        .await
        .unwrap();

    let updated = repository.updated();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].id(), product.id());
    assert_eq!(updated[0].name(), &new_name);
}

#[tokio::test]
async fn it_publishes_a_product_renamed_event() {
    let product = ProductMother::random();
    let repository = Arc::new(ProductRepositoryMock::new().finding(&product));
    let event_bus = Arc::new(EventBusMock::new());

    renamer(&repository, &event_bus)
        .execute(product.id().clone(), ProductNameMother::random())
        .await
        .unwrap();

    assert_eq!(event_bus.published_event_names(), vec![ProductRenamedEvent::EVENT_NAME]);
}

#[tokio::test]
async fn it_publishes_nothing_when_the_name_does_not_change() {
    let name = ProductNameMother::random();
    let product = ProductMother::with_name(name.clone());
    let repository = Arc::new(ProductRepositoryMock::new().finding(&product));
    let event_bus = Arc::new(EventBusMock::new());

    renamer(&repository, &event_bus)
        .execute(product.id().clone(), name)
        .await
        .unwrap();

    assert!(event_bus.published_event_names().is_empty());
}

#[tokio::test]
async fn it_fails_with_not_found_when_the_product_does_not_exist() {
    let id = ProductIdMother::random();
    let repository = Arc::new(ProductRepositoryMock::new());
    let event_bus = Arc::new(EventBusMock::new());

    let result = renamer(&repository, &event_bus)
        .execute(id.clone(), ProductNameMother::random())
        .await;

    assert!(matches!(result, Err(ProductError::NotFound(missing)) if missing == id.value()));
    assert!(repository.updated().is_empty());
    assert!(event_bus.published_event_names().is_empty());
}

#[tokio::test]
async fn it_publishes_nothing_when_the_update_fails() {
    let product = ProductMother::random();
    let repository = Arc::new(ProductRepositoryMock::new().finding(&product).failing_on_update());
    let event_bus = Arc::new(EventBusMock::new());

    let result = renamer(&repository, &event_bus)
        .execute(product.id().clone(), ProductNameMother::random())
        .await;

    assert!(result.is_err());
    assert!(event_bus.published_event_names().is_empty());
}
```

Every test has three blocks separated by a blank line: arrange (mothers and mocks), act (one call), assert. Its name states the behaviour: `it_<does_something>[_when_<condition>]`. There is one test file per application service.

Every command use case gets the same set of tests:

| Test | Asserts |
|---|---|
| The change happens | The mock received the right aggregate, with the right values |
| The event is published | `EventBusMock` received exactly the expected event names |
| No-op changes publish nothing | Nothing published when the aggregate recorded nothing |
| Each domain error propagates | `NotFound`, `AlreadyExists`... reach the caller unchanged |
| A failed write publishes nothing | Nothing published when the port fails |

A query use case asserts the returned value, and the error for a missing aggregate.

---

## Unit tests of the domain

- **Value Objects:** one test per invariant, including the absent case for nullable VOs ([Value Objects](value-objects.md)). They need no doubles.
- **Aggregates:** each behaviour changes the state and records its event, and a no-op records nothing ([Domain Events](domain-events.md#testing)). They are built with mothers and need no doubles.
- **Domain services:** the real service on mocked ports.

```rust
// tests/libs/catalog/src/product/domain/services/product_finder_tests.rs

#[tokio::test]
async fn it_fails_with_not_found_when_no_product_has_the_id() {
    let finder = ProductFinder::new(Arc::new(ProductRepositoryMock::new()));

    let result = finder.find(&ProductIdMother::random()).await;

    assert!(matches!(result, Err(ProductError::NotFound(_))));
}
```

---

## Adapter tests: one contract, every adapter

Every adapter of a port must behave the same way, or end-to-end tests (which use the in-memory adapter) would prove nothing about production (which uses Postgres). Write the expected behaviour once, as a **contract** of plain async functions that take the port, and run it against every adapter:

```rust
// tests/libs/catalog/src/product/infrastructure/persistence/product_repository_contract.rs

pub async fn it_finds_a_saved_product(repository: Arc<dyn ProductRepository>) {
    let product = ProductMother::random();

    repository.save(&product).await.unwrap();

    let found = repository.search(product.id()).await.unwrap().expect("product not found");
    assert_eq!(found.id(), product.id());
    assert_eq!(found.name(), product.name());
}

pub async fn it_finds_nothing_for_an_unknown_id(repository: Arc<dyn ProductRepository>) {
    let found = repository.search(&ProductIdMother::random()).await.unwrap();

    assert!(found.is_none());
}

pub async fn it_rejects_a_duplicated_id(repository: Arc<dyn ProductRepository>) {
    let product = ProductMother::random();
    repository.save(&product).await.unwrap();

    let result = repository.save(&ProductMother::with_id(product.id().clone())).await;

    assert!(matches!(result, Err(ProductError::AlreadyExists(_))));
}
```

```rust
// tests/libs/catalog/src/product/infrastructure/persistence/in_memory_product_repository_tests.rs

#[tokio::test]
async fn it_finds_a_saved_product() {
    contract::it_finds_a_saved_product(Arc::new(InMemoryProductRepository::new())).await;
}
```

```rust
// tests/libs/catalog/src/product/infrastructure/persistence/postgres_product_repository_tests.rs

async fn postgres() -> Arc<dyn ProductRepository> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must point to a test database");
    let pool = PgPool::connect(&url).await.expect("cannot connect to the test database");
    Arc::new(PostgresProductRepository::new(pool))
}

#[tokio::test]
#[ignore = "needs a Postgres test database in DATABASE_URL"]
async fn it_finds_a_saved_product() {
    contract::it_finds_a_saved_product(postgres().await).await;
}
```

Tests that need real infrastructure are `#[ignore]`d, so `cargo test` stays fast and self-contained. CI runs them with a database: `DATABASE_URL=... cargo test -- --ignored`. Because the mothers generate random ids, these tests can share one database without cleaning it between runs.

---

## End-to-end tests

They go through the app's real routes and its real composition root (`compose()`, which wires in-memory adapters). Primitive mothers fill the request bodies:

```rust
// tests/apps/catalog_api/src/product/create_product_test.rs

#[actix_web::test]
async fn it_creates_a_product_and_finds_it() {
    let buses = compose();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(buses.command_bus.clone()))
            .app_data(web::Data::from(buses.query_bus.clone()))
            .service(create_product)
            .service(find_product),
    )
    .await;
    let id = UuidMother::random();
    let name = WordMother::random();

    let created = test::TestRequest::post()
        .uri("/products")
        .set_json(json!({ "id": id, "name": name }))
        .send_request(&app)
        .await;
    assert_eq!(created.status(), 201);

    let found = test::TestRequest::get()
        .uri(&format!("/products/{id}"))
        .send_request(&app)
        .await;
    assert_eq!(found.status(), 200);
    let body: serde_json::Value = test::read_body_json(found).await;
    assert_eq!(body["name"], name);
}
```

Each test calls `compose()`, so each test gets fresh in-memory state and they can run in parallel. For each endpoint, cover the success status and every error status the controller maps: 400 for invalid input, 404, 409, and so on. Business rules are covered by the unit tests. End-to-end tests prove the wiring: routes, (de)serialization, error mapping and the composition root.

---

## Anti-patterns

| Anti-pattern | Fix |
|---|---|
| Mocking aggregates, Value Objects or domain services | Use the real ones; mock only ports |
| A mock that ignores its arguments | Match on them (`search` finds by id), or record them and assert |
| Asserting only call counts | Assert what the mock received (`updated()[0].name()`) |
| A mock that re-implements storage | Use the in-memory adapter where real behaviour is wanted |
| Literal values everywhere (`"p-1"`, `"Chair"`) | Mothers; literals only for the values the test is about |
| Building VOs or aggregates by hand in tests | Their mothers |
| A mother that returns `Result` | `expect` inside: invalid input to a mother is a bug in the test |
| An aggregate mother that uses `create` | `reconstitute`, so no pending events leak into the test |
| `with_<field>` taking a primitive | Take the VO; only the VO mother converts primitives |
| One mother for several types, or mothers in one big file | One mother per type, mirroring the source path |
| `with_a_and_b_and_c` methods multiplying | Turn that mother into a builder |
| Shared state between tests (a `static` repository, one app for all tests) | Each test builds its mocks or calls `compose()` |
| Unit tests that need a database | Mark them as `#[ignore]`d integration tests; unit tests stay in memory |
| Testing in-memory and Postgres adapters with different tests | One contract, run against every adapter |

---

## Checklist

- [ ] Unit tests replace only the ports, with hand-written mocks
- [ ] Each mock is `<Port>Mock`, defaults to the happy path, is configured with `self`-returning methods, and exposes what it received
- [ ] Mocks of shared ports and primitive mothers live in `libs/shared/testing`
- [ ] Each VO has a mother with `create(value)`, `random()`, and named constructors when tests need them
- [ ] Each aggregate has a mother with `create(...)`, `random()`, and `with_<field>(vo)` as tests need them, built with `reconstitute`
- [ ] Mothers mirror the source path of the type they build
- [ ] Tests write literals only for the values they are about
- [ ] Each command use case is tested for its effect, its event, no-op, each error, and no event on failure
- [ ] Every adapter of a port passes the same contract tests; real-infrastructure tests are `#[ignore]`d
- [ ] End-to-end tests call `compose()` per test and cover each endpoint's success and error statuses
