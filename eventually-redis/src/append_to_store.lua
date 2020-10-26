local stream_type = KEYS[1]
local source_id = KEYS[2]
local expected_version = tonumber(ARGV[1])

local source_stream = string.format("%s.%s", stream_type, source_id)
local current_version = tonumber(redis.call("XLEN", source_stream))
local last_version = current_version
local next_sequence_number = redis.call("XLEN", stream_type)

-- Perform optimistic concurrency check over the expected version
-- specified by the client.
if expected_version > -1 and expected_version ~= current_version then
    return redis.error_reply("expected version: " .. expected_version .. ", current version: " .. current_version)
end

-- Insert all the events passed to the script.
for i, event in pairs({unpack(ARGV, 2)}) do
    local version = current_version + i                   -- Versioning starts from 1.
    local sequence_number = next_sequence_number + i - 1  -- Sequence number from 0.

    -- First, it adds the event to the source-related event stream.
    redis.call(
        "XADD",
        source_stream,
        -- Use <version>-<seq.no> format to allow for XRANGE to work
        -- using the version number.
        string.format("%d-%d", version, sequence_number),
        "event", event
    )

    -- Second, it adds the event to the $all event stream.
    redis.call(
        "XADD",
        stream_type,
        -- Use <seq.no>-<version> format to allow for XRANGE to work
        -- using the sequence number.
        string.format("%d-%d", sequence_number, version),
        "source_id", source_id,
        "event", event
    )

    -- Publish the message of the newly-added event for interested subscribers.
    -- Since "PUBLISH" only works with strings, format in JSON string.
    redis.call(
        "PUBLISH",
        stream_type,
        string.format(
            "{\"source_id\":\"%s\",\"sequence_number\":%d,\"version\":%d,\"event\":%s}",
            source_id, sequence_number, version, event -- event is supposed to be JSON already.
        )
    )

    last_version = version
end

-- Return the latest version computed for the source.
return last_version
