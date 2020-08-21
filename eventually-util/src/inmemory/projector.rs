use std::error::Error as StdError;
use std::sync::Arc;

use eventually_core::projection::Projection;
use eventually_core::subscription::Subscription;

use futures::stream::StreamExt;

use crate::sync::RwLock;

/// A `Projector` manages the state of a single [`Projection`]
/// by opening a long-running stream of all events coming from the [`EventStore`].
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
    // Create a new Projector from the provided [`Projection`] and
    // [`Subscription`] values.
    //
    // [`Projection`]: ../../eventually-core/projection/trait.Projection.html
    // [`Subscription`]: ../../eventually-core/subscription/trait.Subscription.html
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
        let mut stream = self.subscription.resume().await?;

        while let Some(result) = stream.next().await {
            println!("Got new event for {}", std::any::type_name::<P>());

            let event = result?;
            let sequence_number = event.sequence_number();

            println!(
                "Project event {} for {}",
                sequence_number,
                std::any::type_name::<P>()
            );

            self.projection
                .write()
                .await
                .project(event)
                .await
                .map_err(anyhow::Error::from)?;

            println!(
                "Update subscription for {} for {}",
                sequence_number,
                std::any::type_name::<P>()
            );

            self.subscription
                .checkpoint(sequence_number)
                .await
                .map_err(anyhow::Error::from)?;
        }

        Ok(())
    }
}
