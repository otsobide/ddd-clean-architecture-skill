// Template → libs/{{context}}/src/{{aggregate}}/application/create_{{aggregate}}/{{aggregate}}_creator.rs
// Reference: references/domain-events.md#publishing-from-the-application-service

use std::sync::Arc;

use shared_domain_events::domain::event_bus::EventBus;

use crate::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use crate::{{aggregate}}::domain::repositories::{{aggregate}}_repository::{{Aggregate}}Repository;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

pub struct {{Aggregate}}Creator {
    repository: Arc<dyn {{Aggregate}}Repository>,
    event_bus: Arc<dyn EventBus>,
}

impl {{Aggregate}}Creator {
    pub fn new(repository: Arc<dyn {{Aggregate}}Repository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self { repository, event_bus }
    }

    pub async fn execute(&self, id: {{Aggregate}}Id, {{concept}}: {{Aggregate}}{{Concept}}) -> Result<(), {{Aggregate}}Error> {
        let mut {{aggregate}} = {{Aggregate}}::create(id, {{concept}});

        self.repository.save(&{{aggregate}}).await?; // persist first
        self.event_bus.publish({{aggregate}}.pull_domain_events()).await; // publish second

        Ok(())
    }
}
