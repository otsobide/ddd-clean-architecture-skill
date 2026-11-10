// Template → libs/{{context}}/src/{{aggregate}}/application/find_{{aggregate}}/find_{{aggregate}}_response.rs
// Reference: references/cqrs.md#query-use-case-example

/// Plain DTO: primitives only, no domain types.
pub struct Find{{Aggregate}}Response {
    pub id: String,
    pub {{concept}}: String,
}
