// Template → libs/{{context}}/src/{{aggregate}}/domain/events/{{aggregate}}_created_event.rs
// Reference: references/domain-events.md#defining-an-event
//
// Move it to the context's shared module as soon as another module subscribes to it.

use std::time::SystemTime;

use shared_domain_events::domain::domain_event::DomainEvent;
use shared_domain_events::domain::domain_event_metadata::DomainEventMetadata;

/// Primitive payload only: subscribers build their own Value Objects.
pub struct {{Aggregate}}CreatedEvent {
    metadata: DomainEventMetadata,
    pub {{concept}}: String,
}

impl {{Aggregate}}CreatedEvent {
    pub const EVENT_NAME: &'static str = "{{context}}.{{aggregate}}.created";

    pub fn new({{aggregate}}_id: impl Into<String>, {{concept}}: impl Into<String>) -> Self {
        Self {
            metadata: DomainEventMetadata::new({{aggregate}}_id),
            {{concept}}: {{concept}}.into(),
        }
    }
}

impl DomainEvent for {{Aggregate}}CreatedEvent {
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
