CREATE OR REPLACE FUNCTION append_to_store(
    aggregate_type_id     TEXT,
    aggregate_id          TEXT,
    current_version       INTEGER,
    perform_version_check BOOLEAN,
    events                JSONB[]
) RETURNS TABLE (
    "version"       INTEGER,
    sequence_number BIGINT
) AS $$
DECLARE
    aggregate_version INTEGER;
    sequence_number   BIGINT;
    "event"           JSONB;
BEGIN

    -- Retrieve the global offset value from the aggregate_type.
    SELECT "offset" INTO sequence_number FROM aggregate_types WHERE id = aggregate_type_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'invalid aggregate type provided: %', aggregate_type_id;
    END IF;

    -- Retrieve the latest aggregate version for the specified aggregate id.
    SELECT aggregates."version" INTO aggregate_version FROM aggregates WHERE id = aggregate_id;
    IF NOT FOUND THEN
        -- Add the initial aggregate representation inside the `aggregates` table.
        INSERT INTO aggregates (id, aggregate_type_id)
        VALUES (aggregate_id, aggregate_type_id);

        -- Make sure to initialize the aggregate version in case
        aggregate_version = 0;
    END IF;

    -- Perform optimistic concurrency check.
    IF perform_version_check AND aggregate_version <> current_version THEN
        RAISE EXCEPTION 'invalid aggregate version provided: %, expected: %', current_version, aggregate_version;
    END IF;

    FOREACH "event" IN ARRAY events
    LOOP
        -- Increment the aggregate version prior to inserting the new event.
        aggregate_version = aggregate_version + 1;
        -- Increment the new sequence number value.
        sequence_number  = sequence_number + 1;

        -- Insert the event into the events table.
        -- Version numbers should start from 1; sequence numbers should start from 0.
        INSERT INTO events (aggregate_id, "version", sequence_number, "event")
        VALUES (aggregate_id, aggregate_version, sequence_number, "event");
    END LOOP;

    -- Update the aggregate with the latest version computed.
    UPDATE aggregates SET "version" = aggregate_version WHERE id = aggregate_id;

    -- Update the global offset with the latest sequence number.
    UPDATE aggregate_types SET "offset" = sequence_number WHERE id = aggregate_type_id;

    RETURN QUERY
        SELECT aggregate_version, sequence_number;

END;
$$ LANGUAGE PLPGSQL;
