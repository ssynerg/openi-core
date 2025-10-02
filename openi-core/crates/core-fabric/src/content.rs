use std::fmt;

/// Simple content-type newtype so we can impl Display and validation later.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentType(pub String);

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ContentType {
    fn from(s: &str) -> Self { ContentType(s.to_string()) }
}
