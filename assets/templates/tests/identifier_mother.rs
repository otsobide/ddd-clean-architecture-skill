// Template → tests/libs/{{context}}/src/{{aggregate}}/domain/value_objects/mothers/{{aggregate}}_id_mother.rs
// Reference: references/testing.md#value-object-mothers

use shared_testing::mothers::uuid_mother::UuidMother;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_id::{{Aggregate}}Id;

pub struct {{Aggregate}}IdMother;

#[allow(dead_code)]
impl {{Aggregate}}IdMother {
    pub fn create(value: impl Into<String>) -> {{Aggregate}}Id {
        {{Aggregate}}Id::new(value).expect("{{Aggregate}}IdMother: invalid {{aggregate}} id")
    }

    pub fn random() -> {{Aggregate}}Id {
        Self::create(UuidMother::random())
    }
}
