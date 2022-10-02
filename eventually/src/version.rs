pub type Version = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("conflict error detected, expected version was: {expected}, found: {actual}")]
pub struct ConflictError {
    pub expected: Version,
    pub actual: Version,
}
