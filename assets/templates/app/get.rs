// Template → apps/{{app}}/src/{{aggregate}}/get.rs
// Reference: references/cqrs.md#dispatching-from-an-entry-point

use actix_web::{get, web, HttpResponse};
use serde_json::json;
use shared_cqrs::query::domain::query_bus::QueryBus;
use shared_cqrs::query::domain::query_bus_error::QueryBusError;

use {{context}}::{{aggregate}}::application::find_{{aggregate}}::find_{{aggregate}}_query::Find{{Aggregate}}Query;
use {{context}}::{{aggregate}}::application::find_{{aggregate}}::find_{{aggregate}}_response::Find{{Aggregate}}Response;
use {{context}}::{{aggregate}}::domain::errors::{{aggregate}}_error::{{Aggregate}}Error;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

#[get("/{{aggregates}}/{id}")]
pub async fn find_{{aggregate}}(query_bus: web::Data<dyn QueryBus>, path: web::Path<String>) -> HttpResponse {
    let id = match {{Aggregate}}Id::new(path.into_inner()) {
        Ok(id) => id,
        Err(error) => return HttpResponse::BadRequest().body(error.to_string()),
    };

    match query_bus.ask(Box::new(Find{{Aggregate}}Query { id })).await {
        Ok(response) => {
            let {{aggregate}} = response
                .downcast::<Find{{Aggregate}}Response>()
                .expect("Find{{Aggregate}}Query is answered with Find{{Aggregate}}Response");
            HttpResponse::Ok().json(json!({ "id": {{aggregate}}.id, "{{concept}}": {{aggregate}}.{{concept}} }))
        }
        Err(QueryBusError::Handler(error)) => match error.downcast_ref::<{{Aggregate}}Error>() {
            Some({{Aggregate}}Error::NotFound(_)) => HttpResponse::NotFound().body(error.to_string()),
            _ => HttpResponse::InternalServerError().body(error.to_string()),
        },
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
