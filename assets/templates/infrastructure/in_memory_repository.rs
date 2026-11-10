// Template → libs/{{context}}/src/{{aggregate}}/infrastructure/persistence/in_memory/in_memory_{{aggregate}}_repository.rs
// Reference: references/hexagonal-architecture.md#rust-traits

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use crate::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use crate::{{aggregate}}::domain::repositories::{{aggregate}}_repository::{{Aggregate}}Repository;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

/// Stores the state, never the aggregate (it carries its pending events).
/// Used by end-to-end tests and local development.
#[derive(Default)]
pub struct InMemory{{Aggregate}}Repository {
    {{aggregates}}: Mutex<HashMap<String, ({{Aggregate}}Id, {{Aggregate}}{{Concept}})>>,
}

impl InMemory{{Aggregate}}Repository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl {{Aggregate}}Repository for InMemory{{Aggregate}}Repository {
    async fn save(&self, {{aggregate}}: &{{Aggregate}}) -> Result<(), {{Aggregate}}Error> {
        let mut {{aggregates}} = self.{{aggregates}}.lock().expect("{{aggregates}} lock poisoned");
        let key = {{aggregate}}.id().value().to_string();
        if {{aggregates}}.contains_key(&key) {
            return Err({{Aggregate}}Error::AlreadyExists(key));
        }
        {{aggregates}}.insert(key, ({{aggregate}}.id().clone(), {{aggregate}}.{{concept}}().clone()));
        Ok(())
    }

    async fn update(&self, {{aggregate}}: &{{Aggregate}}) -> Result<(), {{Aggregate}}Error> {
        let mut {{aggregates}} = self.{{aggregates}}.lock().expect("{{aggregates}} lock poisoned");
        let key = {{aggregate}}.id().value().to_string();
        if !{{aggregates}}.contains_key(&key) {
            return Err({{Aggregate}}Error::NotFound(key));
        }
        {{aggregates}}.insert(key, ({{aggregate}}.id().clone(), {{aggregate}}.{{concept}}().clone()));
        Ok(())
    }

    async fn search(&self, id: &{{Aggregate}}Id) -> Result<Option<{{Aggregate}}>, {{Aggregate}}Error> {
        let {{aggregates}} = self.{{aggregates}}.lock().expect("{{aggregates}} lock poisoned");
        Ok({{aggregates}}
            .get(id.value())
            .map(|(id, {{concept}})| {{Aggregate}}::reconstitute(id.clone(), {{concept}}.clone())))
    }
}
