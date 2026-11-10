// Template → libs/{{context}}/src/{{aggregate}}/domain/value_objects/{{aggregate}}_{{concept}}.rs
// Reference: references/value-objects.md

use shared_valueobject::domain::errors::value_object_validation_error::ValueObjectValidationError;

/// TODO: what this value means in the domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct {{Aggregate}}{{Concept}}(String);

impl {{Aggregate}}{{Concept}} {
    const MAX_LENGTH: usize = 100; // TODO: the domain's limit

    pub fn new(value: impl Into<String>) -> Result<Self, ValueObjectValidationError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(ValueObjectValidationError::new(
                "{{aggregate}} {{concept}} cannot be empty".to_string(),
            ));
        }

        if value.chars().count() > Self::MAX_LENGTH {
            return Err(ValueObjectValidationError::new(format!(
                "{{aggregate}} {{concept}} cannot exceed {} characters",
                Self::MAX_LENGTH
            )));
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
