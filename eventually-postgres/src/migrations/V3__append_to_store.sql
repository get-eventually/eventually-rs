CREATE OR REPLACE PROCEDURE append_to_store(
    aggregate_type     TEXT,
    aggregate_id          TEXT,
    current_version       INTEGER,
    perform_version_check BOOLEAN,
    events                JSONB[],
    INOUT aggregate_version INTEGER DEFAULT NULL,
    INOUT sequence_number BIGINT DEFAULT NULL
) AS $$
DECLARE
    "event"           JSONB;
BEGIN

    -- Retrieve the global offset value from the aggregate_type.
    PERFORM FROM aggregate_types WHERE id = aggregate_type;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'invalid aggregate type provided: %', aggregate_type;
    END IF;

    -- Retrieve the latest aggregate version for the specified aggregate id.
    SELECT aggregates."version" INTO aggregate_version FROM aggregates WHERE id = aggregate_id AND aggregate_type_id = aggregate_type;
    IF NOT FOUND THEN
        -- Add the initial aggregate representation inside the `aggregates` table.
        INSERT INTO aggregates (id, aggregate_type_id)
        VALUES (aggregate_id, aggregate_type);

        -- Make sure to initialize the aggregate version in case
        aggregate_version = 0;
    END IF;

    -- Perform optimistic concurrency check.
    IF perform_version_check AND aggregate_version <> current_version THEN
        RAISE EXCEPTION 'invalid aggregate version provided: %, expected: %', current_version, aggregate_version;
    END IF;

    SELECT last_value INTO sequence_number from events_number_seq;

    FOREACH "event" IN ARRAY events
    LOOP
        -- Increment the aggregate version prior to inserting the new event.
        aggregate_version = aggregate_version + 1;

        -- Insert the event into the events table.
        -- Version numbers should start from 1; sequence numbers should start from 0.
        INSERT INTO events (aggregate_id, aggregate_type, "version", "event")
        VALUES (aggregate_id, aggregate_type, aggregate_version, "event")
        RETURNING events.sequence_number INTO sequence_number;

        -- Send a notification to all listeners of the newly added events.
        PERFORM pg_notify(aggregate_type, ''
            || '{'
            || '"source_id": "'      || aggregate_id      || '" ,'
            || '"version": '         || aggregate_version || ', '
            || '"sequence_number": ' || sequence_number   || ', '
            || '"event": '           || "event"::TEXT
            || '}');
        COMMIT;

    END LOOP;

    -- Update the aggregate with the latest version computed.
    UPDATE aggregates SET "version" = aggregate_version WHERE id = aggregate_id AND aggregate_type_id = aggregate_type;

    -- Update the global offset with the latest sequence number.
    UPDATE aggregate_types SET "offset" = sequence_number WHERE id = aggregate_type;

END;
$$ LANGUAGE PLPGSQL;
