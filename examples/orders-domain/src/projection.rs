use futures::future;
use futures::future::{BoxFuture, FutureExt};

use serde::Serialize;

use eventually::store::Persisted;
use eventually::Projection;

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct TotalOrdersProjection {
    created: u64,
    completed: u64,
    cancelled: u64,
}

impl Projection for TotalOrdersProjection {
    type SourceId = String;
    type Event = crate::OrderEvent;
    type Error = std::convert::Infallible;

    fn project<'a>(
        &'a mut self,
        event: Persisted<Self::SourceId, Self::Event>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        use crate::OrderEvent;

        match event.take() {
            OrderEvent::Created { .. } => self.created += 1,
            OrderEvent::Completed { .. } => self.completed += 1,
            OrderEvent::Cancelled { .. } => self.cancelled += 1,
            _ => (),
        };

        future::ok(()).boxed()
    }
}
