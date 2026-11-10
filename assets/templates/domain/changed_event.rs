// Template → libs/{{context}}/src/{{aggregate}}/domain/events/{{aggregate}}_{{past}}_event.rs
// Reference: references/domain-events.md#defining-an-event
//
// Move it to the context's shared module as soon as another module subscribes to it.

use std::time::SystemTime;

use shared_domain_events::domain::domain_event::DomainEvent;
use shared_domain_events::domain::domain_event_metadata::DomainEventMetadata;

/// Carries both sides of the change, as primitives.
pub struct {{Aggregate}}{{Past}}Event {
    metadata: DomainEventMetadata,
    pub previous_{{concept}}: String,
    pub {{concept}}: String,
}

impl {{Aggregate}}{{Past}}Event {
    pub const EVENT_NAME: &'static str = "{{context}}.{{aggregate}}.{{past}}";

    pub fn new(
        {{aggregate}}_id: impl Into<String>,
        previous_{{concept}}: impl Into<String>,
        {{concept}}: impl Into<String>,
    ) -> Self {
        Self {
            metadata: DomainEventMetadata::new({{aggregate}}_id),
            previous_{{concept}}: previous_{{concept}}.into(),
            {{concept}}: {{concept}}.into(),
        }
    }
}

impl DomainEvent for {{Aggregate}}{{Past}}Event {
    fn event_name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn event_id(&self) -> &str {
        &self.metadata.event_id
    }

    fn aggregate_id(&self) -> &str {
        &self.metadata.aggregate_id
    }

    fn occurred_on(&self) -> SystemTime {
        self.metadata.occurred_on
    }
}
