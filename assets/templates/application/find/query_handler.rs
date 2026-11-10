// Template → libs/{{context}}/src/{{aggregate}}/application/find_{{aggregate}}/find_{{aggregate}}_query_handler.rs
// Reference: references/cqrs.md#query-use-case-example

use async_trait::async_trait;
use shared_cqrs::query::domain::query_handler::QueryHandler;

use super::find_{{aggregate}}_query::Find{{Aggregate}}Query;
use super::find_{{aggregate}}_response::Find{{Aggregate}}Response;
use super::{{aggregate}}_finder::{{Aggregate}}Finder;
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;

pub struct Find{{Aggregate}}QueryHandler {
    finder: {{Aggregate}}Finder,
}

impl Find{{Aggregate}}QueryHandler {
    pub fn new(finder: {{Aggregate}}Finder) -> Self {
        Self { finder }
    }
}

#[async_trait]
impl QueryHandler<Find{{Aggregate}}Query> for Find{{Aggregate}}QueryHandler {
    type Response = Find{{Aggregate}}Response;
    type Error = {{Aggregate}}Error;

    async fn handle(&self, query: Find{{Aggregate}}Query) -> Result<Find{{Aggregate}}Response, {{Aggregate}}Error> {
        let {{aggregate}} = self.finder.execute(query.id).await?;

        Ok(Find{{Aggregate}}Response {
            id: {{aggregate}}.id().value().to_string(),
            {{concept}}: {{aggregate}}.{{concept}}().value().to_string(),
        })
    }
}
