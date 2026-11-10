// Template → tests/libs/{{context}}/src/{{aggregate}}/domain/entities/mothers/{{aggregate}}_mother.rs
// Reference: references/testing.md#aggregate-mothers

use {{context}}::{{aggregate}}::domain::entities::{{aggregate}}::{{Aggregate}};
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

use crate::src::{{aggregate}}::domain::value_objects::mothers::{{aggregate}}_id_mother::{{Aggregate}}IdMother;
use crate::src::{{aggregate}}::domain::value_objects::mothers::{{aggregate}}_{{concept}}_mother::{{Aggregate}}{{Concept}}Mother;

pub struct {{Aggregate}}Mother;

#[allow(dead_code)]
impl {{Aggregate}}Mother {
    /// Every value given. Built with `reconstitute`: no pending events leak into the test.
    pub fn create(id: {{Aggregate}}Id, {{concept}}: {{Aggregate}}{{Concept}}) -> {{Aggregate}} {
        {{Aggregate}}::reconstitute(id, {{concept}})
    }

    /// Every value random.
    pub fn random() -> {{Aggregate}} {
        Self::create({{Aggregate}}IdMother::random(), {{Aggregate}}{{Concept}}Mother::random())
    }

    /// Random, except the id.
    pub fn with_id(id: {{Aggregate}}Id) -> {{Aggregate}} {
        Self::create(id, {{Aggregate}}{{Concept}}Mother::random())
    }

    /// Random, except the {{concept}}.
    pub fn with_{{concept}}({{concept}}: {{Aggregate}}{{Concept}}) -> {{Aggregate}} {
        Self::create({{Aggregate}}IdMother::random(), {{concept}})
    }
}
