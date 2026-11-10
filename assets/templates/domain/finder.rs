// Template → libs/{{context}}/src/{{aggregate}}/domain/services/{{aggregate}}_finder.rs
// Reference: references/hexagonal-architecture.md#when-a-use-case-needs-another-use-case-domain-services
//
// Domain service shared by every use case that loads an existing {{aggregate}}.
// No buses, no events, no DTOs. Handlers never call it: application services do.

use std::sync::Arc;

use crate::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use crate::{{aggregate}}::domain::repositories::{{aggregate}}_repository::{{Aggregate}}Repository;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

/// Finds a {{aggregate}} or fails with `{{Aggregate}}Error::NotFound`.
pub struct {{Aggregate}}Finder {
    repository: Arc<dyn {{Aggregate}}Repository>,
}

impl {{Aggregate}}Finder {
    pub fn new(repository: Arc<dyn {{Aggregate}}Repository>) -> Self {
        Self { repository }
    }

    pub async fn find(&self, id: &{{Aggregate}}Id) -> Result<{{Aggregate}}, {{Aggregate}}Error> {
        self.repository
            .search(id)
            .await?
            .ok_or_else(|| {{Aggregate}}Error::NotFound(id.value().to_string()))
    }
}
