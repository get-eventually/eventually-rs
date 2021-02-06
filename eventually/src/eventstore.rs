use futures::future::BoxFuture;
use futures::stream::BoxStream;

pub type EventStream<'a, T, E> = BoxStream<'a, Result<PersistedEvent<T>, E>>;

pub trait EventStore: Sync + Send {
    type Event: Send + Sync;
    type AppendError: IntoConflictError + Send + Sync;
    type StreamError: Send + Sync;

    fn append<'a>(
        &'a mut self,
        stream: StreamInstance<'a>,
        expected: VersionCheck,
        events: Vec<Self::Event>,
    ) -> BoxFuture<'a, Result<u64, Self::AppendError>>;

    fn stream(
        &self,
        stream: StreamName,
        select: Select,
    ) -> EventStream<Self::Event, Self::StreamError>;

    fn subscribe(stream: StreamName) -> EventStream<Self::Event, Self::StreamError>;
}

pub struct StreamInstance<'a>(pub &'a str, pub &'a str);

impl<'a> StreamInstance<'a> {
    #[inline]
    pub fn typ(&self) -> &str {
        self.0
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.1
    }
}

impl<'a> From<StreamInstance<'a>> for StreamName<'a> {
    #[inline]
    fn from(instance: StreamInstance<'a>) -> Self {
        Self::Instance(instance)
    }
}

pub enum Select {
    All,
    From(u64),
}

pub enum StreamName<'a> {
    All,
    Type(&'a str),
    Instance(StreamInstance<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum VersionCheck {
    Any,
    Exact(u64),
}

impl VersionCheck {
    #[inline]
    pub fn check(&self, current_version: u64) -> Result<(), ConflictError> {
        match *self {
            VersionCheck::Any => Ok(()),
            VersionCheck::Exact(expected) if current_version == expected => Ok(()),
            VersionCheck::Exact(expected) => Err(ConflictError {
                expected,
                actual: current_version,
            }),
        }
    }
}

pub trait IntoConflictError {
    fn into_conflict_error(&self) -> Option<ConflictError>;
}

impl IntoConflictError for std::convert::Infallible {
    #[inline]
    fn into_conflict_error(&self) -> Option<ConflictError> {
        None
    }
}

impl IntoConflictError for ConflictError {
    #[inline]
    fn into_conflict_error(&self) -> Option<ConflictError> {
        Some(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("conflict error: expected stream version {expected}, actual {actual}")]
pub struct ConflictError {
    pub expected: u64,
    pub actual: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedEvent<T> {
    pub stream_type: String,
    pub stream_name: String,
    pub version: u64,
    pub event: T,
}
