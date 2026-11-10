// Template → libs/{{context}}/src/{{module}}/application/{{reaction}}_on_{{aggregate}}_{{past}}/{{reaction}}_on_{{aggregate}}_{{past}}.rs
// Reference: references/domain-events.md#subscribers
//
// Lives in the module that reacts ({{module}}). The event and the id it carries are used by two
// modules, so they live in the context's shared module.

use async_trait::async_trait;
use shared_domain_events::domain::domain_event::DomainEvent;
use shared_domain_events::domain::domain_event_subscriber::DomainEventSubscriber;

use super::{{reaction_service}}::{{ReactionService}};
use crate::shared::domain::events::{{aggregate}}_{{past}}_event::{{Aggregate}}{{Past}}Event;
use crate::shared::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{module}}::domain::errors::{{module}}_error::{{Module}}Error;

/// Thin: turns the event's primitives into Value Objects and delegates to an application
/// service of this module (never a domain service). Idempotency lives in that service.
pub struct {{Reaction}}On{{Aggregate}}{{Past}} {
    service: {{ReactionService}},
}

impl {{Reaction}}On{{Aggregate}}{{Past}} {
    pub fn new(service: {{ReactionService}}) -> Self {
        Self { service }
    }
}

#[async_trait]
impl DomainEventSubscriber<{{Aggregate}}{{Past}}Event> for {{Reaction}}On{{Aggregate}}{{Past}} {
    type Error = {{Module}}Error;

    async fn on(&self, event: &{{Aggregate}}{{Past}}Event) -> Result<(), {{Module}}Error> {
        // Build this module's Value Objects from the event's primitives
        // ({{Module}}Error implements From<ValueObjectValidationError>).
        // Pass `event.event_id()` too when the service deduplicates by it.
        let {{aggregate}}_id = {{Aggregate}}Id::new(event.aggregate_id())?;

        self.service.execute({{aggregate}}_id).await
    }
}
