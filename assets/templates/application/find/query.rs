// Template → libs/{{context}}/src/{{aggregate}}/application/find_{{aggregate}}/find_{{aggregate}}_query.rs
// Reference: references/cqrs.md#query-use-case-example

use shared_cqrs::query::domain::query::Query;

use crate::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

pub struct Find{{Aggregate}}Query {
    pub id: {{Aggregate}}Id,
}

impl Query for Find{{Aggregate}}Query {}
