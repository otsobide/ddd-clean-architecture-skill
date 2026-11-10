// Template → tests/libs/{{context}}/src/{{aggregate}}/application/{{verb}}_{{aggregate}}/{{aggregate}}_{{verber}}_tests.rs
// Reference: references/testing.md#unit-tests-of-application-services

use std::sync::Arc;

use shared_testing::mocks::event_bus_mock::EventBusMock;
use {{context}}::{{aggregate}}::application::{{verb}}_{{aggregate}}::{{aggregate}}_{{verber}}::{{Aggregate}}{{Verber}};
use {{context}}::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use {{context}}::{{aggregate}}::domain::events::{{aggregate}}_{{past}}_event::{{Aggregate}}{{Past}}Event;
use {{context}}::{{aggregate}}::domain::services::{{aggregate}}_finder::{{Aggregate}}Finder;

use crate::src::mocks::{{aggregate}}_repository_mock::{{Aggregate}}RepositoryMock;
use crate::src::{{aggregate}}::domain::entities::mothers::{{aggregate}}_mother::{{Aggregate}}Mother;
use crate::src::{{aggregate}}::domain::value_objects::mothers::{{aggregate}}_id_mother::{{Aggregate}}IdMother;
use crate::src::{{aggregate}}::domain::value_objects::mothers::{{aggregate}}_{{concept}}_mother::{{Aggregate}}{{Concept}}Mother;

/// Small composition root: the real domain service on top of the mocked port.
fn {{verber}}(
    repository: &Arc<{{Aggregate}}RepositoryMock>,
    event_bus: &Arc<EventBusMock>,
) -> {{Aggregate}}{{Verber}} {
    {{Aggregate}}{{Verber}}::new(
        {{Aggregate}}Finder::new(repository.clone()),
        repository.clone(),
        event_bus.clone(),
    )
}

#[tokio::test]
async fn it_{{verb}}s_an_existing_{{aggregate}}() {
    let {{aggregate}} = {{Aggregate}}Mother::random();
    let new_{{concept}} = {{Aggregate}}{{Concept}}Mother::random();
    let repository = Arc::new({{Aggregate}}RepositoryMock::new().finding(&{{aggregate}}));
    let event_bus = Arc::new(EventBusMock::new());

    {{verber}}(&repository, &event_bus)
        .execute({{aggregate}}.id().clone(), new_{{concept}}.clone())
        .await
        .unwrap();

    let updated = repository.updated();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].id(), {{aggregate}}.id());
    assert_eq!(updated[0].{{concept}}(), &new_{{concept}});
}

#[tokio::test]
async fn it_publishes_a_{{aggregate}}_{{past}}_event() {
    let {{aggregate}} = {{Aggregate}}Mother::random();
    let repository = Arc::new({{Aggregate}}RepositoryMock::new().finding(&{{aggregate}}));
    let event_bus = Arc::new(EventBusMock::new());

    {{verber}}(&repository, &event_bus)
        .execute({{aggregate}}.id().clone(), {{Aggregate}}{{Concept}}Mother::random())
        .await
        .unwrap();

    assert_eq!(event_bus.published_event_names(), vec![{{Aggregate}}{{Past}}Event::EVENT_NAME]);
}

#[tokio::test]
async fn it_publishes_nothing_when_the_{{concept}}_does_not_change() {
    let {{concept}} = {{Aggregate}}{{Concept}}Mother::random();
    let {{aggregate}} = {{Aggregate}}Mother::with_{{concept}}({{concept}}.clone());
    let repository = Arc::new({{Aggregate}}RepositoryMock::new().finding(&{{aggregate}}));
    let event_bus = Arc::new(EventBusMock::new());

    {{verber}}(&repository, &event_bus)
        .execute({{aggregate}}.id().clone(), {{concept}})
        .await
        .unwrap();

    assert!(event_bus.published_event_names().is_empty());
}

#[tokio::test]
async fn it_fails_with_not_found_when_the_{{aggregate}}_does_not_exist() {
    let id = {{Aggregate}}IdMother::random();
    let repository = Arc::new({{Aggregate}}RepositoryMock::new());
    let event_bus = Arc::new(EventBusMock::new());

    let result = {{verber}}(&repository, &event_bus)
        .execute(id.clone(), {{Aggregate}}{{Concept}}Mother::random())
        .await;

    assert!(matches!(result, Err({{Aggregate}}Error::NotFound(missing)) if missing == id.value()));
    assert!(repository.updated().is_empty());
    assert!(event_bus.published_event_names().is_empty());
}

#[tokio::test]
async fn it_publishes_nothing_when_the_update_fails() {
    let {{aggregate}} = {{Aggregate}}Mother::random();
    let repository = Arc::new({{Aggregate}}RepositoryMock::new().finding(&{{aggregate}}).failing_on_update());
    let event_bus = Arc::new(EventBusMock::new());

    let result = {{verber}}(&repository, &event_bus)
        .execute({{aggregate}}.id().clone(), {{Aggregate}}{{Concept}}Mother::random())
        .await;

    assert!(result.is_err());
    assert!(event_bus.published_event_names().is_empty());
}
