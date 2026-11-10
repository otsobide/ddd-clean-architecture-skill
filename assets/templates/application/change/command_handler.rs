// Template → libs/{{context}}/src/{{aggregate}}/application/{{verb}}_{{aggregate}}/{{verb}}_{{aggregate}}_command_handler.rs
// Reference: references/cqrs.md#handlers-never-call-domain-services

use async_trait::async_trait;
use shared_cqrs::command::domain::command_handler::CommandHandler;

use super::{{aggregate}}_{{verber}}::{{Aggregate}}{{Verber}};
use super::{{verb}}_{{aggregate}}_command::{{Verb}}{{Aggregate}}Command;
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;

pub struct {{Verb}}{{Aggregate}}CommandHandler {
    {{verber}}: {{Aggregate}}{{Verber}}, // the application service, never a domain service
}

impl {{Verb}}{{Aggregate}}CommandHandler {
    pub fn new({{verber}}: {{Aggregate}}{{Verber}}) -> Self {
        Self { {{verber}} }
    }
}

#[async_trait]
impl CommandHandler<{{Verb}}{{Aggregate}}Command> for {{Verb}}{{Aggregate}}CommandHandler {
    type Error = {{Aggregate}}Error;

    async fn handle(&self, command: {{Verb}}{{Aggregate}}Command) -> Result<(), {{Aggregate}}Error> {
        self.{{verber}}.execute(command.id, command.{{concept}}).await
    }
}
