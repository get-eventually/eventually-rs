use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::Add,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64);

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl Add<u64> for Version {
    type Output = Self;

    fn add(self, other: u64) -> Self::Output {
        Self(self.0 + other)
    }
}

impl Add for Version {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        self + other.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceNumber(u64);

impl Display for SequenceNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("conflict error detected, expected event stream version was: {expected}, found: {actual}")]
pub struct ConflictError {
    pub expected: Version,
    pub actual: Version,
}

pub trait ToConflictError {
    fn to_conflict_error(&self) -> Option<ConflictError> {
        None
    }
}

impl ToConflictError for ConflictError {
    fn to_conflict_error(&self) -> Option<ConflictError> {
        Some(*self)
    }
}

impl ToConflictError for std::convert::Infallible {}
