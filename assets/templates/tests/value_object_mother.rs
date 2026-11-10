// Template → tests/libs/{{context}}/src/{{aggregate}}/domain/value_objects/mothers/{{aggregate}}_{{concept}}_mother.rs
// Reference: references/testing.md#value-object-mothers

use shared_testing::mothers::word_mother::WordMother;
use {{context}}::{{aggregate}}::domain::value_objects::{{aggregate}}_{{concept}}::{{Aggregate}}{{Concept}};

pub struct {{Aggregate}}{{Concept}}Mother;

#[allow(dead_code)]
impl {{Aggregate}}{{Concept}}Mother {
    /// A concrete value: only for values the test is about.
    pub fn create(value: impl Into<String>) -> {{Aggregate}}{{Concept}} {
        {{Aggregate}}{{Concept}}::new(value).expect("{{Aggregate}}{{Concept}}Mother: invalid {{aggregate}} {{concept}}")
    }

    /// Any valid value.
    pub fn random() -> {{Aggregate}}{{Concept}} {
        Self::create(WordMother::random())
    }

    // TODO: named constructors with domain meaning when tests need them (`longest()`, `empty()`...).
}
