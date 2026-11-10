# Value Objects

A **Value Object (VO)** is a small, immutable type that represents a domain concept defined entirely by its value. It has no identity: two VOs holding the same value are equal and interchangeable.

VOs replace raw primitives (`String`, `i64`, `bool`, `Option<String>`...) inside the domain. Instead of passing a `String` that *should* be a non-empty key, you pass a `ConfigKey` that *cannot* be anything else.

VOs are the building blocks of aggregate roots (see [Aggregate roots](#aggregate-roots-where-value-objects-live)).

Examples use Rust, but the rules apply to any language.

> **Golden rule: an aggregate root never holds a nullable Value Object.**
>
> If a concept is optional, the absence is modelled **inside** the VO: the VO accepts a nullable input (`Option<String>`, `null`, `None`...) and decides what "absent" means. The aggregate always holds a VO instance, never `Option<VO>` / `null`.
>
> See [Nullable concepts](#nullable-concepts-null-lives-inside-the-vo-never-around-it).

---

## Rules

1. **Every aggregate field is a VO.** No raw primitives as fields of entities or aggregates.
2. **Validate at construction.** The constructor returns `Result<Self, ValueObjectValidationError>`. If you hold a VO, it is valid. Invalid state cannot be represented.
3. **Immutable.** Private inner value, no setters, no `pub` fields. To "change" a VO, build a new one.
4. **Equality by value.** Derive `PartialEq`, `Eq` (and `Hash` when used as a key or identifier).
5. **Expose the primitive explicitly.** Provide a `value()` getter. Implement `Display` when the VO has a natural text form.
6. **One VO per file.** `domain/value_objects/config_key.rs` → `struct ConfigKey`.
7. **Generic primitives live in a shared crate.** Context-specific VOs reuse them via the newtype pattern.
8. **Nullability lives inside the VO, never around it.** No `Option<VO>` in aggregates, commands, or queries.
9. **No infrastructure in VOs.** A VO knows nothing about HTTP, SQL, or serialization frameworks.

---

## Basic VO: validated newtype

`ConfigKey` is the identifier of the `ConfigEntry` aggregate. It must not be blank and has a maximum length.

```rust
// libs/config/src/config_entry/domain/value_objects/config_key.rs

use shared_valueobject::domain::errors::value_object_validation_error::ValueObjectValidationError;

/// Unique key that identifies a ConfigEntry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConfigKey(String);

impl ConfigKey {
    const MAX_LENGTH: usize = 128;

    pub fn new(value: impl Into<String>) -> Result<Self, ValueObjectValidationError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(ValueObjectValidationError::new(
                "config key cannot be empty".to_string(),
            ));
        }

        if value.len() > Self::MAX_LENGTH {
            return Err(ValueObjectValidationError::new(format!(
                "config key cannot exceed {} characters",
                Self::MAX_LENGTH
            )));
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

What this buys you:

- A function that receives a `ConfigKey` never has to check for emptiness again.
- `ConfigKey` and `ConfigValue` are different types, so the compiler rejects swapping them by mistake.
- Validation rules for the concept live in exactly one place.

---

## Shared primitives and composition

Generic, context-free rules (for example, "a non-empty string") belong in the shared value object crate. Context-specific VOs wrap them and add their own rules on top.

```rust
// libs/shared/valueobject/src/domain/errors/value_object_validation_error.rs

#[derive(Debug)]
pub struct ValueObjectValidationError(String);

impl ValueObjectValidationError {
    pub fn new(message: String) -> Self {
        Self(message)
    }
}

impl std::fmt::Display for ValueObjectValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValueObject validation error: {}", self.0)
    }
}

impl std::error::Error for ValueObjectValidationError {}
```

```rust
// libs/shared/valueobject/src/domain/strings/string_value_object.rs

use crate::domain::errors::value_object_validation_error::ValueObjectValidationError;

/// Non-empty string primitive, reused by context-specific VOs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StringValueObject(String);

impl StringValueObject {
    pub fn new(value: impl Into<String>) -> Result<Self, ValueObjectValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ValueObjectValidationError::new(
                "value cannot be empty".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
```

```rust
// libs/config/src/config_entry/domain/value_objects/config_value.rs

use shared_valueobject::domain::errors::value_object_validation_error::ValueObjectValidationError;
use shared_valueobject::domain::strings::string_value_object::StringValueObject;

/// Value stored under a ConfigKey.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigValue(StringValueObject);

impl ConfigValue {
    pub fn new(value: impl Into<String>) -> Result<Self, ValueObjectValidationError> {
        Ok(Self(StringValueObject::new(value)?))
    }

    pub fn value(&self) -> &str {
        self.0.value()
    }
}
```

The `?` propagates the shared validation. `ConfigValue` can add more rules before or after it.

---

## Aggregate roots: where Value Objects live

An **aggregate** is a cluster of domain objects treated as a single unit whenever its data changes. Its **aggregate root** is the entity at the top of that cluster, and it is the only way in:

- Outside code references, loads, and saves the aggregate **only through its root**.
- Each aggregate has one repository, and the repository works with whole aggregates.

An aggregate root is an entity, so unlike a VO it has **identity**. Two config entries holding the same value are still different entries if their keys differ. That identity is itself a VO (`ConfigKey`).

| | Value Object | Aggregate root |
|---|---|---|
| Identity | None: defined by its value | Yes: an identifier VO |
| Equality | By value | By identifier |
| Mutability | Immutable | Its state changes over time, through its own methods |
| Lifecycle | Created and discarded freely | Created, saved, loaded, modified, and deleted through its repository |
| Made of | Primitives (or other VOs) | Value Objects (and child entities) |

This section shows the aggregate root only as a **container of Value Objects, with no behaviour**. Behaviour (methods that change state, enforce rules involving several fields, and record domain events) is covered in [Domain Events](domain-events.md).

```rust
// libs/config/src/config_entry/domain/entities/config_entry.rs

/// Aggregate root. Identified by its `ConfigKey`.
#[derive(Clone, Debug)]
pub struct ConfigEntry {
    key: ConfigKey,     // identity, itself a VO
    value: ConfigValue, // every other field is a VO too
}

impl ConfigEntry {
    /// Receives VOs that are already valid, so it has nothing left to validate field by field.
    pub fn new(key: ConfigKey, value: ConfigValue) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &ConfigKey {
        &self.key
    }

    pub fn value(&self) -> &ConfigValue {
        &self.value
    }
}

/// Equality by identity, not by value: same key means same entry.
impl PartialEq for ConfigEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for ConfigEntry {}
```

What the root guarantees about its Value Objects:

- **Every field is a VO.** No raw primitives, and never `Option<VO>` (see [Nullable concepts](#nullable-concepts-null-lives-inside-the-vo-never-around-it)).
- **Private fields, getters that return VOs.** `entry.key()` returns `&ConfigKey`. Callers unwrap to primitives with `.value()` only at the boundaries (DTOs, persistence, event payloads).
- **No setters.** State never changes by assigning a field from outside. Changes come only through the root's own behaviour methods.
- **The identity never changes.** The identifier is set at construction and is never replaced.
- **Single-field rules live in the VOs.** The constructor receives valid VOs, so it never re-checks emptiness, length, or format. The root is only responsible for rules that involve several of its fields together.

---

## Nullable concepts: null lives inside the VO, never around it

Some domain concepts are optional. Say a `ConfigEntry` can have a description, but doesn't have to.

### ❌ Wrong: nullable VO in the aggregate

```rust
pub struct ConfigEntry {
    key: ConfigKey,
    value: ConfigValue,
    description: Option<ConfigDescription>, // ❌ never do this
}
```

Why this is wrong:

- **The meaning of "absent" leaks out of the domain concept.** Is `""` a description or no description? With `Option<VO>`, that decision is made by whoever builds the aggregate (the controller, a mapper, the repository), and each one may decide differently.
- **`None` skips the VO entirely.** Normalization and validation rules never run on the absent path.
- **Every consumer has to branch.** Getters return `Option<&ConfigDescription>`, and use cases, event constructors, mappers, and tests all end up matching on it.
- **It is inconsistent.** Mandatory concepts are VOs, optional concepts are `Option<VO>`. The aggregate's shape now depends on optionality instead of on the domain.

### ✅ Right: the VO accepts a nullable input

```rust
// libs/config/src/config_entry/domain/value_objects/config_description.rs

use shared_valueobject::domain::errors::value_object_validation_error::ValueObjectValidationError;

/// Optional human-readable description of a ConfigEntry.
///
/// Absence is a valid state of this VO, not of the aggregate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigDescription(Option<String>);

impl ConfigDescription {
    const MAX_LENGTH: usize = 512;

    /// Accepts a nullable input. The VO, not the caller, decides what "absent" means.
    pub fn new(value: Option<String>) -> Result<Self, ValueObjectValidationError> {
        match value {
            None => Ok(Self(None)),
            // Domain decision owned by the VO: a blank description means no description.
            Some(v) if v.trim().is_empty() => Ok(Self(None)),
            Some(v) if v.len() > Self::MAX_LENGTH => Err(ValueObjectValidationError::new(
                format!("config description cannot exceed {} characters", Self::MAX_LENGTH),
            )),
            Some(v) => Ok(Self(Some(v))),
        }
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn value(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}
```

```rust
// libs/config/src/config_entry/domain/entities/config_entry.rs

#[derive(Clone)]
pub struct ConfigEntry {
    key: ConfigKey,
    value: ConfigValue,
    description: ConfigDescription, // ✅ always present; it may *contain* nothing
}

impl ConfigEntry {
    pub fn new(key: ConfigKey, value: ConfigValue, description: ConfigDescription) -> Self {
        Self { key, value, description }
    }

    pub fn key(&self) -> &ConfigKey {
        &self.key
    }

    pub fn value(&self) -> &ConfigValue {
        &self.value
    }

    pub fn description(&self) -> &ConfigDescription {
        &self.description
    }
}
```

The same rule applies to commands and queries: they carry `ConfigDescription`, never `Option<ConfigDescription>`.

Domain events are different. Their payload is made of primitives because they leave the domain (see [Domain Events](domain-events.md)), so an optional description travels in an event as `Option<String>`.

### Where nullable primitives are allowed

Nullable **primitives** are fine at the boundaries, where data enters or leaves the domain:

| Layer | Type for an optional description |
|---|---|
| HTTP request / response body | `Option<String>` |
| Response DTO (`ConfigEntryEntry`) | `Option<String>` |
| Persistence row / document | `Option<String>` / `NULL` column |
| Domain event payload | `Option<String>` |
| Command, query | `ConfigDescription` |
| Aggregate / entity | `ConfigDescription` |

**Entering the domain** (HTTP handler builds the command):

```rust
#[derive(Deserialize)]
pub struct CreateConfigEntryRequest {
    pub key: String,
    pub value: String,
    pub description: Option<String>, // nullable primitive at the boundary: OK
}

fn to_command(
    body: &CreateConfigEntryRequest,
) -> Result<CreateConfigEntryCommand, ValueObjectValidationError> {
    Ok(CreateConfigEntryCommand {
        key: ConfigKey::new(body.key.clone())?,
        value: ConfigValue::new(body.value.clone())?,
        description: ConfigDescription::new(body.description.clone())?, // always a VO
    })
}

#[post("/config")]
pub async fn handler(
    state: web::Data<AppState>,
    body: web::Json<CreateConfigEntryRequest>,
) -> impl Responder {
    let command = match to_command(&body) {
        Ok(command) => command,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    // dispatch through the command bus...
}
```

A VO validation failure at the boundary maps to **400 Bad Request**: the input was invalid, and nothing reached the domain.

**Leaving the domain** (query handler maps the aggregate to a DTO):

```rust
ConfigEntryEntry {
    key: entry.key().value().to_string(),
    value: entry.value().value().to_string(),
    description: entry.description().value().map(str::to_string),
}
```

**Reconstituting from persistence** (repository maps a row back to the aggregate):

```rust
let description = ConfigDescription::new(row.description)
    .map_err(|e| ConfigEntryRepositoryError::Unexpected(e.to_string()))?;
```

---

## Identifiers are Value Objects too

Aggregate identifiers are VOs with format validation:

```rust
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValueObjectValidationError> {
        let value = value.into();
        Uuid::parse_str(&value).map_err(|_| {
            ValueObjectValidationError::new(format!("'{value}' is not a valid UUID"))
        })?;
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
```

---

## Testing Value Objects

### Validation tests

Each VO gets its own tests for its invariants, including the nullable path:

```rust
#[test]
fn it_rejects_an_empty_key() {
    assert!(ConfigKey::new("").is_err());
}

#[test]
fn it_rejects_a_key_longer_than_the_maximum() {
    assert!(ConfigKey::new("k".repeat(129)).is_err());
}

#[test]
fn it_accepts_a_missing_description() {
    let description = ConfigDescription::new(None).unwrap();
    assert!(description.is_empty());
}

#[test]
fn it_treats_a_blank_description_as_missing() {
    let description = ConfigDescription::new(Some("   ".to_string())).unwrap();
    assert!(description.is_empty());
}
```

### Object Mothers

Mothers build valid VOs for tests, so tests don't repeat construction noise. A nullable VO's mother exposes both states:

```rust
// tests/libs/config/src/config_entry/domain/value_objects/mothers/config_key_mother.rs

pub struct ConfigKeyMother;

impl ConfigKeyMother {
    pub fn create(value: impl Into<String>) -> ConfigKey {
        ConfigKey::new(value).expect("ConfigKeyMother produced an invalid key")
    }

    pub fn random() -> ConfigKey {
        Self::create(Uuid::new_v4().to_string())
    }
}
```

```rust
// tests/libs/config/src/config_entry/domain/value_objects/mothers/config_description_mother.rs

pub struct ConfigDescriptionMother;

impl ConfigDescriptionMother {
    pub fn create(value: Option<&str>) -> ConfigDescription {
        ConfigDescription::new(value.map(str::to_string))
            .expect("ConfigDescriptionMother produced an invalid description")
    }

    pub fn random() -> ConfigDescription {
        Self::create(Some(&Uuid::new_v4().to_string()))
    }

    pub fn empty() -> ConfigDescription {
        ConfigDescription::empty()
    }
}
```

---

## Anti-patterns

| Anti-pattern | Fix |
|---|---|
| Raw primitives in aggregates (`key: String`) | Wrap them in a VO (`key: ConfigKey`) |
| `Option<VO>` in an aggregate, command, or query | Make the VO accept `Option<T>` and model absence internally |
| Constructor that never fails, with validation done elsewhere (controller, service) | Validate inside `new()` and return `Result` |
| Public inner field (`pub struct ConfigKey(pub String)`) | Private field plus a `value()` getter |
| Setters or `&mut self` methods on a VO | Build a new VO instead |
| One generic VO reused for unrelated concepts (`StringValueObject` as a field type) | One context-specific newtype per concept that wraps the generic one |
| VO depending on HTTP, SQL, or serialization frameworks | Keep VOs pure; map at the boundaries |

---

## Checklist

- [ ] Every aggregate field is a VO
- [ ] The aggregate root has private fields, getters that return VOs, no setters, and equality by identity
- [ ] No `Option<VO>` in aggregates, commands, or queries
- [ ] Optional concepts use a VO whose constructor accepts `Option<T>`
- [ ] The constructor validates and returns `Result<Self, ValueObjectValidationError>`
- [ ] Inner value is private; exposed via `value()`
- [ ] Derives `Clone`, `Debug`, `PartialEq`, `Eq` (`Hash` for identifiers)
- [ ] One VO per file, file name = type name in `snake_case`
- [ ] Unit tests cover each invariant, including the absent case for nullable VOs
- [ ] An Object Mother exists for the VO (`create`, `random`, plus `empty` when nullable)
