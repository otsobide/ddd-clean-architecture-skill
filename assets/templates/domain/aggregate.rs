// Template → libs/{{context}}/src/{{aggregate}}/domain/entities/{{aggregate}}.rs
// Reference: references/value-objects.md#aggregate-roots-where-value-objects-live,
//            references/domain-events.md#the-aggregate-records-its-events

use shared_domain_events::domain::domain_event::DomainEvent;

use crate::{{aggregate}}::domain::events::{{aggregate}}_created_event::{{Aggregate}}CreatedEvent;
use crate::{{aggregate}}::domain::events::{{aggregate}}_{{past}}_event::{{Aggregate}}{{Past}}Event;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

/// Aggregate root. Every field is a Value Object, never `Option<VO>`.
pub struct {{Aggregate}} {
    id: {{Aggregate}}Id,
    {{concept}}: {{Aggregate}}{{Concept}},
    domain_events: Vec<Box<dyn DomainEvent>>,
}

impl {{Aggregate}} {
    /// A brand-new {{aggregate}}: records `{{Aggregate}}CreatedEvent`.
    pub fn create(id: {{Aggregate}}Id, {{concept}}: {{Aggregate}}{{Concept}}) -> Self {
        let mut {{aggregate}} = Self::reconstitute(id, {{concept}});
        {{aggregate}}.record({{Aggregate}}CreatedEvent::new(
            {{aggregate}}.id.value(),
            {{aggregate}}.{{concept}}.value(),
        ));
        {{aggregate}}
    }

    /// An existing {{aggregate}} rebuilt from storage: records nothing.
    pub fn reconstitute(id: {{Aggregate}}Id, {{concept}}: {{Aggregate}}{{Concept}}) -> Self {
        Self { id, {{concept}}, domain_events: Vec::new() }
    }

    /// Behaviour: enforces the rule, changes the state, and records what happened.
    pub fn {{verb}}(&mut self, {{concept}}: {{Aggregate}}{{Concept}}) {
        if self.{{concept}} == {{concept}} {
            return; // nothing changed, nothing happened
        }
        let previous = std::mem::replace(&mut self.{{concept}}, {{concept}});
        self.record({{Aggregate}}{{Past}}Event::new(
            self.id.value(),
            previous.value(),
            self.{{concept}}.value(),
        ));
    }

    /// Hands over the recorded events and clears them, so they are published once.
    pub fn pull_domain_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.domain_events)
    }

    fn record(&mut self, event: impl DomainEvent) {
        self.domain_events.push(Box::new(event));
    }

    pub fn id(&self) -> &{{Aggregate}}Id {
        &self.id
    }

    pub fn {{concept}}(&self) -> &{{Aggregate}}{{Concept}} {
        &self.{{concept}}
    }
}

/// Equality by identity: same id, same {{aggregate}}.
impl PartialEq for {{Aggregate}} {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for {{Aggregate}} {}
