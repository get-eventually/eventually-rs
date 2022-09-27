CREATE TABLE aggregates (
    aggregate_id TEXT    NOT NULL PRIMARY KEY REFERENCES event_streams (event_stream_id) ON DELETE CASCADE,
    "type"       TEXT    NOT NULL,
    "version"    INTEGER NOT NULL CHECK ("version" > 0),
    "state"      BYTEA   NOT NULL
);

CREATE PROCEDURE upsert_aggregate(
    _aggregate_id TEXT,
    _type TEXT,
    _expected_version INTEGER,
    _new_version INTEGER,
    _state BYTEA
)
LANGUAGE PLPGSQL
AS $$
DECLARE
    current_aggregate_version INTEGER;
BEGIN
    -- Retrieve the latest version for the target aggregate.
    SELECT a."version"
    INTO current_aggregate_version
    FROM aggregates a
    WHERE a.aggregate_id = _aggregate_id;

    IF (NOT FOUND AND _expected_version <> 0) OR (current_aggregate_version <> _expected_version)
    THEN
        RAISE EXCEPTION 'aggregate version check failed, expected: %, got: %', _expected_version, current_aggregate_version;
    END IF;

    -- An Aggregate Root is also an Event Stream.
    INSERT INTO event_streams (event_stream_id, "version")
    VALUES (_aggregate_id, _new_version)
    ON CONFLICT (event_stream_id) DO
    UPDATE SET "version" = _new_version;

    INSERT INTO aggregates (aggregate_id, "type", "version", "state")
    VALUES (_aggregate_id, _type, _new_version, _state)
    ON CONFLICT (aggregate_id) DO
    UPDATE SET "version" = _new_version, "state" = _state;
END;
$$;
