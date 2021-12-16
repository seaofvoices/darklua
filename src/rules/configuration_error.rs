use std::fmt;

/// When implementing the configure method of the Rule trait, the method returns a result that uses
/// this error type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RuleConfigurationError {
    /// When a rule gets an unknown property. The string should be the unknown field name.
    UnexpectedProperty(String),
    /// When a rule has a required property. The string should be the field name.
    MissingProperty(String),
    /// When a property is associated with something else than an expected string. The string is
    /// the property name.
    StringExpected(String),
    /// When a property is associated with something else than an expected unsigned number. The
    /// string is the property name.
    UsizeExpected(String),
    /// When a property is associated with something else than an expected list of strings. The
    /// string is the property name.
    StringListExpected(String),
    /// When the value type is invalid. The string is the property name that was given the wrong
    /// value type.
    UnexpectedValueType(String),
}

impl fmt::Display for RuleConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuleConfigurationError::*;

        match self {
            UnexpectedProperty(property) => write!(f, "unexpected field '{}'", property),
            MissingProperty(property) => write!(f, "missing required field '{}'", property),
            StringExpected(property) => write!(f, "string value expected for field '{}'", property),
            UsizeExpected(property) => {
                write!(f, "unsigned integer expected for field '{}'", property)
            }
            StringListExpected(property) => {
                write!(f, "list of string expected for field '{}'", property)
            }
            UnexpectedValueType(property) => write!(f, "unexpected type for field '{}'", property),
        }
    }
}
