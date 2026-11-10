// Template → apps/{{app}}/src/{{aggregate}}/put.rs
// Reference: references/cqrs.md#dispatching-from-an-entry-point

use actix_web::{put, web, HttpResponse};
use serde::Deserialize;
use shared_cqrs::command::domain::command_bus::CommandBus;
use shared_cqrs::command::domain::command_bus_error::CommandBusError;

use {{context}}::{{aggregate}}::application::{{verb}}_{{aggregate}}::{{verb}}_{{aggregate}}_command::{{Verb}}{{Aggregate}}Command;
use {{context}}::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

#[derive(Deserialize)]
pub struct {{Verb}}{{Aggregate}}Request {
    pub {{concept}}: String,
}

#[put("/{{aggregates}}/{id}")]
pub async fn {{verb}}_{{aggregate}}(
    command_bus: web::Data<dyn CommandBus>,
    path: web::Path<String>,
    body: web::Json<{{Verb}}{{Aggregate}}Request>,
) -> HttpResponse {
    let command = match ({{Aggregate}}Id::new(path.into_inner()), {{Aggregate}}{{Concept}}::new(&body.{{concept}})) {
        (Ok(id), Ok({{concept}})) => {{Verb}}{{Aggregate}}Command { id, {{concept}} },
        (Err(error), _) | (_, Err(error)) => return HttpResponse::BadRequest().body(error.to_string()),
    };

    match command_bus.dispatch(Box::new(command)).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(CommandBusError::Handler(error)) => match error.downcast_ref::<{{Aggregate}}Error>() {
            Some({{Aggregate}}Error::NotFound(_)) => HttpResponse::NotFound().body(error.to_string()),
            _ => HttpResponse::InternalServerError().body(error.to_string()),
        },
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
