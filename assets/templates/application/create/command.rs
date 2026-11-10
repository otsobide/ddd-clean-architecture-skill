// Template → libs/{{context}}/src/{{aggregate}}/application/create_{{aggregate}}/create_{{aggregate}}_command.rs
// Reference: references/cqrs.md#command-use-case-example

use shared_cqrs::command::domain::command::Command;

use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

/// Fields are Value Objects; the caller generates the id.
pub struct Create{{Aggregate}}Command {
    pub id: {{Aggregate}}Id,
    pub {{concept}}: {{Aggregate}}{{Concept}},
}

impl Command for Create{{Aggregate}}Command {}
