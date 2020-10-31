use futures::stream::Stream;

use redis::aio::MultiplexedConnection;
use redis::streams::{StreamId, StreamKey, StreamRangeReply, StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, RedisResult};

/// Returns a [`futures::Stream`] instance out of a paginated series of
/// requests to read a Redis Stream using `XRANGE`.
///
/// Each page is as big as `page_size`; for each page requested,
/// all the entries in [`StreamRangeReply`] are yielded in the stream,
/// until the entries are fully exhausted.
///
/// The stream stop when all entries in the Redis Stream have been returned.
///
/// [`futures::Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
/// [`StreamRangeReply`]: https://docs.rs/redis/0.17.0/redis/streams/struct.StreamRangeReply.html
pub fn into_xrange_stream(
    mut conn: MultiplexedConnection,
    stream_name: String,
    page_size: usize,
    from: usize,
) -> impl Stream<Item = RedisResult<StreamId>> + 'static {
    async_stream::try_stream! {
        let mut from = from;

        loop {
            let result: StreamRangeReply = conn
                .xrange_count(&stream_name, from, "+", page_size)
                .await?;

            let ids = result.ids;
            let size = ids.len();

            for id in ids {
                from = parse_version(&id.id) + 1;
                yield id;
            }

            if size < page_size {
                break;
            }
        }
    }
}

/// Returns a long-running [`futures::Stream`] instance returning
/// the results of reading a Redis Stream using Consumer Groups with `XREADGROUP`.
///
/// Each read block should be as big as `page_size`; for each page requested,
/// all the keys in [`StreamReadReply`] are yielded in the stream,
/// until the entries are fully exhausted.
///
/// The stream won't be closed until **program termination** or **explicitly dropped**.
///
/// [`futures::Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
/// [`StreamReadReply`]: https://docs.rs/redis/0.17.0/redis/streams/struct.StreamReadReply.html
pub fn into_xread_stream(
    mut conn: MultiplexedConnection,
    stream_name: String,
    group_name: String,
    page_size: usize,
) -> impl Stream<Item = RedisResult<StreamKey>> + 'static {
    async_stream::try_stream! {
        loop {
            let opts = StreamReadOptions::default()
                .count(page_size)
                // TODO: should the consumer name be configurable?
                .group(&group_name, "eventually-consumer");

            let result: StreamReadReply = conn
                .xread_options(&[&stream_name], &[">"], opts)
                .await?;

            for key in result.keys {
                yield key;
            }
        }
    }
}

/// Parses the version component from the Entry ID of a Redis Stream entry.
pub(crate) fn parse_version(id: &str) -> usize {
    let parts: Vec<&str> = id.split('-').collect();
    parts[0].parse().unwrap()
}
