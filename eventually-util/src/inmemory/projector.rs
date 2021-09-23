use std::error::Error as StdError;
use std::sync::Arc;

use eventually_core::projection::Projection;
use eventually_core::subscription::Subscription;

use futures::stream::StreamExt;
use futures::TryFutureExt;

use crate::sync::RwLock;

/// A `Projector` manages the state of a single [`Projection`]
/// by opening a long-running stream of all events coming from the
/// [`EventStore`].
///
/// New instances of a `Projector` are obtainable through a [`ProjectorBuilder`]
/// instance.
///
/// The `Projector` will start updating the [`Projection`] state when [`run`]
/// is called.
///
/// At each update, the `Projector` will broadcast the latest version of the
/// [`Projection`] on a `Stream` obtainable through [`watch`].
///
/// [`Projection`]: ../../../eventually-core/projection/trait.Projection.html
/// [`EventStore`]: ../../../eventually-core/store/trait.EventStore.html
/// [`ProjectorBuilder`]: struct.ProjectorBuilder.html
/// [`run`]: struct.Projector.html#method.run
/// [`watch`]: struct.Projector.html#method.watch
pub struct Projector<P, S>
where
    P: Projection,
{
    projection: Arc<RwLock<P>>,
    subscription: S,
}

impl<P, S> Projector<P, S>
where
    P: Projection,
    <P as Projection>::SourceId: std::fmt::Debug,
    <P as Projection>::Event: std::fmt::Debug,
    S: Subscription<SourceId = P::SourceId, Event = P::Event>,
    // NOTE: these bounds are needed for anyhow::Error conversion.
    <P as Projection>::Error: StdError + Send + Sync + 'static,
    <S as Subscription>::Error: StdError + Send + Sync + 'static,
{
    /// Create a new Projector from the provided [`Projection`] and
    /// [`Subscription`] values.
    ///
    /// [`Projection`]: ../../eventually-core/projection/trait.Projection.html
    /// [`Subscription`]:
    /// ../../eventually-core/subscription/trait.Subscription.html
    pub fn new(projection: Arc<RwLock<P>>, subscription: S) -> Self {
        Self {
            projection,
            subscription,
        }
    }

    /// Starts the update of the `Projection` by processing all the events
    /// coming from the [`EventStore`].
    ///
    /// [`EventStore`]: ../../../eventually-core/store/trait.EventStore.html
    pub async fn run(&mut self) -> anyhow::Result<()> {
        #[cfg(feature = "with-tracing")]
        let projection_type = std::any::type_name::<P>();

        let mut stream = self.subscription.resume();

        while let Some(result) = stream.next().await {
            let event = result?;
            let sequence_number = event.sequence_number();

            #[cfg(feature = "with-tracing")]
            tracing::debug!(
                sequence_number = sequence_number,
                projection_type = projection_type,
                event = ?event,
                "Projecting new event",
            );

            self.projection
                .write()
                .await
                .project(event)
                .inspect_ok(|_| {
                    #[cfg(feature = "with-tracing")]
                    tracing::debug!(
                        sequence_number = sequence_number,
                        projection_type = projection_type,
                        "Projection succeeded"
                    );
                })
                .inspect_err(
                    #[allow(unused_variables)]
                    {
                        |e| {
                            #[cfg(feature = "with-tracing")]
                            tracing::error!(
                                error = %e,
                                sequence_number = sequence_number,
                                projection_type = projection_type,
                                "Projection failed"
                            )
                        }
                    },
                )
                .await
                .map_err(anyhow::Error::from)?;

            self.subscription
                .checkpoint(sequence_number)
                .inspect_ok(|_| {
                    #[cfg(feature = "with-tracing")]
                    tracing::debug!(
                        sequence_number = sequence_number,
                        projection_type = projection_type,
                        "Subscription checkpointed"
                    );
                })
                .inspect_err(
                    #[allow(unused_variables)]
                    {
                        |e| {
                            #[cfg(feature = "with-tracing")]
                            tracing::error!(
                                error = %e,
                                sequence_number = sequence_number,
                                projection_type = projection_type,
                                "Failed to checkpoint subscription"
                            )
                        }
                    },
                )
                .await
                .map_err(anyhow::Error::from)?;
        }

        Ok(())
    }
}
