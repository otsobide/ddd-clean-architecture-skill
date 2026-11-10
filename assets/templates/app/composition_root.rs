// Template → apps/{{app}}/src/lib.rs
// Reference: references/dependency-injection.md#the-composition-root

pub mod {{aggregate}};

use std::sync::Arc;

use actix_web::web;
use shared_cqrs::command::domain::command_bus::CommandBus;
use shared_cqrs::command::infrastructure::in_memory::in_memory_command_bus::InMemoryCommandBus;
use shared_cqrs::query::domain::query_bus::QueryBus;
use shared_cqrs::query::infrastructure::in_memory::in_memory_query_bus::InMemoryQueryBus;
use shared_domain_events::infrastructure::in_memory::in_memory_event_bus::InMemoryEventBus;

use {{context}}::{{aggregate}}::application::create_{{aggregate}}::create_{{aggregate}}_command_handler::Create{{Aggregate}}CommandHandler;
use {{context}}::{{aggregate}}::application::create_{{aggregate}}::{{aggregate}}_creator::{{Aggregate}}Creator;
use {{context}}::{{aggregate}}::application::find_{{aggregate}}::find_{{aggregate}}_query_handler::Find{{Aggregate}}QueryHandler;
use {{context}}::{{aggregate}}::application::find_{{aggregate}}::{{aggregate}}_finder::{{Aggregate}}Finder;
use {{context}}::{{aggregate}}::application::{{verb}}_{{aggregate}}::{{aggregate}}_{{verber}}::{{Aggregate}}{{Verber}};
use {{context}}::{{aggregate}}::application::{{verb}}_{{aggregate}}::{{verb}}_{{aggregate}}_command_handler::{{Verb}}{{Aggregate}}CommandHandler;
use {{context}}::{{aggregate}}::domain::repositories::{{aggregate}}_repository::{{Aggregate}}Repository;
use {{context}}::{{aggregate}}::domain::services::{{aggregate}}_finder::{{Aggregate}}Finder as Domain{{Aggregate}}Finder;
use {{context}}::{{aggregate}}::infrastructure::persistence::in_memory::in_memory_{{aggregate}}_repository::InMemory{{Aggregate}}Repository;

/// What the entry points get: only the command and query buses.
#[derive(Clone)]
pub struct Buses {
    pub command_bus: Arc<dyn CommandBus>,
    pub query_bus: Arc<dyn QueryBus>,
}

pub fn compose() -> Buses {
    // 1. Adapters (TODO: choose production adapters from configuration)
    let {{aggregate}}_repository: Arc<dyn {{Aggregate}}Repository> = Arc::new(InMemory{{Aggregate}}Repository::new());

    // 2. Buses: created empty and shared before anything uses them
    let command_bus = Arc::new(InMemoryCommandBus::new());
    let query_bus = Arc::new(InMemoryQueryBus::new());
    let event_bus = Arc::new(InMemoryEventBus::new());

    // 3. Application services, each with only what it uses
    let {{aggregate}}_creator = {{Aggregate}}Creator::new({{aggregate}}_repository.clone(), event_bus.clone());
    let {{aggregate}}_{{verber}} = {{Aggregate}}{{Verber}}::new(
        Domain{{Aggregate}}Finder::new({{aggregate}}_repository.clone()),
        {{aggregate}}_repository.clone(),
        event_bus.clone(),
    );
    let {{aggregate}}_finder = {{Aggregate}}Finder::new(Domain{{Aggregate}}Finder::new({{aggregate}}_repository.clone()));

    // 4. Handlers and subscribers, registered last
    command_bus
        .register(Create{{Aggregate}}CommandHandler::new({{aggregate}}_creator))
        .expect("Create{{Aggregate}}CommandHandler registered twice");
    command_bus
        .register({{Verb}}{{Aggregate}}CommandHandler::new({{aggregate}}_{{verber}}))
        .expect("{{Verb}}{{Aggregate}}CommandHandler registered twice");
    query_bus
        .register(Find{{Aggregate}}QueryHandler::new({{aggregate}}_finder))
        .expect("Find{{Aggregate}}QueryHandler registered twice");

    Buses { command_bus, query_bus }
}

/// Registers the routes and hands the buses to the controllers:
/// `App::new().configure(|cfg| configure(cfg, &buses))`.
pub fn configure(cfg: &mut web::ServiceConfig, buses: &Buses) {
    cfg.app_data(web::Data::from(buses.command_bus.clone()))
        .app_data(web::Data::from(buses.query_bus.clone()))
        .service({{aggregate}}::post::create_{{aggregate}})
        .service({{aggregate}}::put::{{verb}}_{{aggregate}})
        .service({{aggregate}}::get::find_{{aggregate}});
}
