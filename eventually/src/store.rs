use std::future::Future;

use futures::Stream;

pub trait ReadStore {
    type SourceId: Eq;
    type Offset: PartialOrd;
    type Event;
    type Stream: Stream<Item = Self::Event>;

    fn stream(&self, source_id: Self::SourceId, from: Self::Offset) -> Self::Stream;
}

pub trait WriteStore: ReadStore {
    type Error;
    type Result: Future<Output = Result<(), Self::Error>>;

    fn append(
        &mut self,
        source_id: Self::SourceId,
        from: Self::Offset,
        events: Vec<Self::Event>,
    ) -> Self::Result;
}
