// Template → libs/{{context}}/src/{{aggregate}}/application/create_{{aggregate}}/create_{{aggregate}}_command_handler.rs
// Reference: references/cqrs.md#command-use-case-example

use async_trait::async_trait;
use shared_cqrs::command::domain::command_handler::CommandHandler;

use super::create_{{aggregate}}_command::Create{{Aggregate}}Command;
use super::{{aggregate}}_creator::{{Aggregate}}Creator;
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;

/// Thin: unpacks the command and delegates to the application service.
pub struct Create{{Aggregate}}CommandHandler {
    creator: {{Aggregate}}Creator,
}

impl Create{{Aggregate}}CommandHandler {
    pub fn new(creator: {{Aggregate}}Creator) -> Self {
        Self { creator }
    }
}

#[async_trait]
impl CommandHandler<Create{{Aggregate}}Command> for Create{{Aggregate}}CommandHandler {
    type Error = {{Aggregate}}Error;

    async fn handle(&self, command: Create{{Aggregate}}Command) -> Result<(), {{Aggregate}}Error> {
        self.creator.execute(command.id, command.{{concept}}).await
    }
}
