// Template → tests/libs/{{context}}/src/mocks/{{aggregate}}_repository_mock.rs
// Reference: references/testing.md#mocks

use std::sync::Mutex;

use async_trait::async_trait;

use {{context}}::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use {{context}}::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use {{context}}::{{aggregate}}::domain::repositories::{{aggregate}}_repository::{{Aggregate}}Repository;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

/// Hand-written mock of the `{{Aggregate}}Repository` port.
/// Stub: what `search` finds and which writes fail. Spy: what was saved and updated.
#[derive(Default)]
pub struct {{Aggregate}}RepositoryMock {
    stored: Vec<{{Aggregate}}>,
    save_fails: bool,
    update_fails: bool,
    saved: Mutex<Vec<{{Aggregate}}>>,
    updated: Mutex<Vec<{{Aggregate}}>>,
}

#[allow(dead_code)]
impl {{Aggregate}}RepositoryMock {
    /// Finds nothing, and every write succeeds.
    pub fn new() -> Self {
        Self::default()
    }

    /// `search` finds a copy of this {{aggregate}}, by its id.
    pub fn finding(mut self, {{aggregate}}: &{{Aggregate}}) -> Self {
        self.stored.push(copy_of({{aggregate}}));
        self
    }

    /// `save` fails with `AlreadyExists`.
    pub fn failing_on_save(mut self) -> Self {
        self.save_fails = true;
        self
    }

    /// `update` fails with `Unexpected`.
    pub fn failing_on_update(mut self) -> Self {
        self.update_fails = true;
        self
    }

    pub fn saved(&self) -> Vec<{{Aggregate}}> {
        self.saved.lock().unwrap().iter().map(copy_of).collect()
    }

    pub fn updated(&self) -> Vec<{{Aggregate}}> {
        self.updated.lock().unwrap().iter().map(copy_of).collect()
    }
}

#[async_trait]
impl {{Aggregate}}Repository for {{Aggregate}}RepositoryMock {
    async fn save(&self, {{aggregate}}: &{{Aggregate}}) -> Result<(), {{Aggregate}}Error> {
        if self.save_fails {
            return Err({{Aggregate}}Error::AlreadyExists({{aggregate}}.id().value().to_string()));
        }
        self.saved.lock().unwrap().push(copy_of({{aggregate}}));
        Ok(())
    }

    async fn update(&self, {{aggregate}}: &{{Aggregate}}) -> Result<(), {{Aggregate}}Error> {
        if self.update_fails {
            return Err({{Aggregate}}Error::Unexpected("update failed (mock)".to_string()));
        }
        self.updated.lock().unwrap().push(copy_of({{aggregate}}));
        Ok(())
    }

    async fn search(&self, id: &{{Aggregate}}Id) -> Result<Option<{{Aggregate}}>, {{Aggregate}}Error> {
        Ok(self.stored.iter().find(|{{aggregate}}| {{aggregate}}.id() == id).map(copy_of))
    }
}

/// `{{Aggregate}}` isn't `Clone` (it owns its pending events), so the mock keeps
/// copies of its state, as a real repository would.
fn copy_of({{aggregate}}: &{{Aggregate}}) -> {{Aggregate}} {
    {{Aggregate}}::reconstitute({{aggregate}}.id().clone(), {{aggregate}}.{{concept}}().clone())
}
