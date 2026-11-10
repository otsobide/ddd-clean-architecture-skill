// Template → apps/{{app}}/src/{{aggregate}}/post.rs
// Reference: references/cqrs.md#dispatching-from-an-entry-point

use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use shared_cqrs::command::domain::command_bus::CommandBus;
use shared_cqrs::command::domain::command_bus_error::CommandBusError;

use {{context}}::{{aggregate}}::application::create_{{aggregate}}::create_{{aggregate}}_command::Create{{Aggregate}}Command;
use {{context}}::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

#[derive(Deserialize)]
pub struct Create{{Aggregate}}Request {
    pub id: String,
    pub {{concept}}: String,
}

#[post("/{{aggregates}}")]
pub async fn create_{{aggregate}}(
    command_bus: web::Data<dyn CommandBus>,
    body: web::Json<Create{{Aggregate}}Request>,
) -> HttpResponse {
    // Invalid input never reaches the domain: 400.
    let command = match ({{Aggregate}}Id::new(&body.id), {{Aggregate}}{{Concept}}::new(&body.{{concept}})) {
        (Ok(id), Ok({{concept}})) => Create{{Aggregate}}Command { id, {{concept}} },
        (Err(error), _) | (_, Err(error)) => return HttpResponse::BadRequest().body(error.to_string()),
    };

    match command_bus.dispatch(Box::new(command)).await {
        Ok(()) => HttpResponse::Created().finish(),
        Err(CommandBusError::Handler(error)) => match error.downcast_ref::<{{Aggregate}}Error>() {
            Some({{Aggregate}}Error::AlreadyExists(_)) => HttpResponse::Conflict().body(error.to_string()),
            _ => HttpResponse::InternalServerError().body(error.to_string()),
        },
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
