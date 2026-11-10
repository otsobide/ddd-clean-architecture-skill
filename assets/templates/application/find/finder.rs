// Template → libs/{{context}}/src/{{aggregate}}/application/find_{{aggregate}}/{{aggregate}}_finder.rs
// Reference: references/cqrs.md#handlers-never-call-domain-services
//
// Application wrapper around the domain finder: the query handler calls this, never the domain one.

use crate::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use crate::{{aggregate}}::domain::services::{{aggregate}}_finder::{{Aggregate}}Finder as Domain{{Aggregate}}Finder;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

pub struct {{Aggregate}}Finder {
    finder: Domain{{Aggregate}}Finder,
}

impl {{Aggregate}}Finder {
    pub fn new(finder: Domain{{Aggregate}}Finder) -> Self {
        Self { finder }
    }

    /// Returns the aggregate. Mapping to a DTO is the handler's job.
    pub async fn execute(&self, id: {{Aggregate}}Id) -> Result<{{Aggregate}}, {{Aggregate}}Error> {
        self.finder.find(&id).await
    }
}
