// Template → libs/{{context}}/src/{{aggregate}}/domain/errors/{{aggregate}}_error.rs
// Reference: references/hexagonal-architecture.md#rust-traits

use thiserror::Error;

/// Every error the {{aggregate}} module's ports and use cases can return.
/// Adapters translate technology errors into these variants.
#[derive(Debug, Error)]
pub enum {{Aggregate}}Error {
    #[error("{{aggregate}} {0} not found")]
    NotFound(String),

    #[error("{{aggregate}} {0} already exists")]
    AlreadyExists(String),

    #[error("unexpected error: {0}")]
    Unexpected(String),
}
