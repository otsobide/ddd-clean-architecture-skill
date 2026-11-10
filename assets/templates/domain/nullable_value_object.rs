// Template → libs/{{context}}/src/{{aggregate}}/domain/value_objects/{{aggregate}}_{{concept}}.rs
// Reference: references/value-objects.md#nullable-concepts-null-lives-inside-the-vo-never-around-it
//
// For optional concepts. The aggregate holds this VO, never `Option<{{Aggregate}}{{Concept}}>`.

use shared_valueobject::domain::errors::value_object_validation_error::ValueObjectValidationError;

/// TODO: what this value means in the domain. Absence is a valid state of this VO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct {{Aggregate}}{{Concept}}(Option<String>);

impl {{Aggregate}}{{Concept}} {
    const MAX_LENGTH: usize = 1000; // TODO: the domain's limit

    /// Accepts a nullable input. The VO, not the caller, decides what "absent" means.
    pub fn new(value: Option<String>) -> Result<Self, ValueObjectValidationError> {
        match value {
            None => Ok(Self(None)),
            Some(v) if v.trim().is_empty() => Ok(Self(None)), // TODO: confirm blank means absent
            Some(v) if v.chars().count() > Self::MAX_LENGTH => Err(ValueObjectValidationError::new(
                format!("{{aggregate}} {{concept}} cannot exceed {} characters", Self::MAX_LENGTH),
            )),
            Some(v) => Ok(Self(Some(v))),
        }
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn value(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}
