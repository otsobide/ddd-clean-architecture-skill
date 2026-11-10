// Template → libs/{{context}}/src/{{aggregate}}/application/{{verb}}_{{aggregate}}/{{verb}}_{{aggregate}}_command.rs
// Reference: references/cqrs.md#command-use-case-example

use shared_cqrs::command::domain::command::Command;

use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

pub struct {{Verb}}{{Aggregate}}Command {
    pub id: {{Aggregate}}Id,
    pub {{concept}}: {{Aggregate}}{{Concept}},
}

impl Command for {{Verb}}{{Aggregate}}Command {}
