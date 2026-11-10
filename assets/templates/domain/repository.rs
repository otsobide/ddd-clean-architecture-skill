// Template → libs/{{context}}/src/{{aggregate}}/domain/repositories/{{aggregate}}_repository.rs
// Reference: references/hexagonal-architecture.md#rust-traits

use async_trait::async_trait;

use crate::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

/// Port: the collection of `{{Aggregate}}` aggregates. Domain types only.
/// Implemented in `infrastructure/persistence/`.
#[async_trait]
pub trait {{Aggregate}}Repository: Send + Sync {
    /// Stores a new {{aggregate}}. Fails with `AlreadyExists` if the id is taken.
    async fn save(&self, {{aggregate}}: &{{Aggregate}}) -> Result<(), {{Aggregate}}Error>;

    /// Replaces a stored {{aggregate}}. Fails with `NotFound` if there is none.
    async fn update(&self, {{aggregate}}: &{{Aggregate}}) -> Result<(), {{Aggregate}}Error>;

    /// Returns the {{aggregate}}, or `None` if no {{aggregate}} has this id.
    async fn search(&self, id: &{{Aggregate}}Id) -> Result<Option<{{Aggregate}}>, {{Aggregate}}Error>;
}
