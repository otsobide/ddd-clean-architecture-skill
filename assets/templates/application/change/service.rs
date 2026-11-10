// Template → libs/{{context}}/src/{{aggregate}}/application/{{verb}}_{{aggregate}}/{{aggregate}}_{{verber}}.rs
// Reference: references/hexagonal-architecture.md#use-cases

use std::sync::Arc;

use shared_domain_events::domain::event_bus::EventBus;

use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use crate::{{aggregate}}::domain::repositories::{{aggregate}}_repository::{{Aggregate}}Repository;
use crate::{{aggregate}}::domain::services::{{aggregate}}_finder::{{Aggregate}}Finder;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

pub struct {{Aggregate}}{{Verber}} {
    finder: {{Aggregate}}Finder,                     // domain service
    repository: Arc<dyn {{Aggregate}}Repository>, // port
    event_bus: Arc<dyn EventBus>,               // port
}

impl {{Aggregate}}{{Verber}} {
    pub fn new(
        finder: {{Aggregate}}Finder,
        repository: Arc<dyn {{Aggregate}}Repository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self { finder, repository, event_bus }
    }

    pub async fn execute(&self, id: {{Aggregate}}Id, {{concept}}: {{Aggregate}}{{Concept}}) -> Result<(), {{Aggregate}}Error> {
        // 1. Load through the domain service (fails with NotFound).
        let mut {{aggregate}} = self.finder.find(&id).await?;

        // 2. The aggregate applies the change and records the event.
        {{aggregate}}.{{verb}}({{concept}});

        // 3. Persist, 4. then publish what was recorded.
        self.repository.update(&{{aggregate}}).await?;
        self.event_bus.publish({{aggregate}}.pull_domain_events()).await;

        Ok(())
    }
}
