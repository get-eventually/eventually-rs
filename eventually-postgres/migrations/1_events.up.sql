CREATE TABLE event_streams (
    event_stream_id TEXT    NOT NULL PRIMARY KEY,
    "version"       INTEGER NOT NULL CHECK ("version" > 0)
);

CREATE TABLE events (
    event_stream_id  TEXT    NOT NULL,
    "type"           TEXT    NOT NULL,
    "version"        INTEGER NOT NULL CHECK ("version" > 0),
    "event"          BYTEA   NOT NULL,
    metadata         JSONB,

    PRIMARY KEY (event_stream_id, "version"),
    FOREIGN KEY (event_stream_id) REFERENCES event_streams (event_stream_id) ON DELETE CASCADE
);

CREATE INDEX event_stream_id_idx ON events (event_stream_id);

CREATE PROCEDURE upsert_event_stream(
    _event_stream_id TEXT,
    _expected_version INTEGER,
    _new_version INTEGER
)
LANGUAGE PLPGSQL
AS $$
DECLARE
    current_event_stream_version INTEGER;
BEGIN
    -- Retrieve the latest version for the target Event Stream.
    SELECT es."version"
    INTO current_event_stream_version
    FROM event_streams es
    WHERE es.event_stream_id = _event_stream_id;

    IF (NOT FOUND AND _expected_version <> 0) OR (current_event_stream_version <> _expected_version)
    THEN
        RAISE EXCEPTION 'event stream version check failed, expected: %, got: %', _expected_version, current_event_stream_version;
    END IF;

    INSERT INTO event_streams (event_stream_id, "version")
    VALUES (_event_stream_id, _new_version)
    ON CONFLICT (event_stream_id) DO
    UPDATE SET "version" = _new_version;
END;
$$;

CREATE FUNCTION upsert_event_stream_with_no_version_check(
    _event_stream_id TEXT,
    _new_version_offset INTEGER
)
RETURNS INTEGER
LANGUAGE PLPGSQL
AS $$
DECLARE
    current_event_stream_version INTEGER;
    new_event_stream_version INTEGER;
BEGIN
    -- Retrieve the latest version for the target Event Stream.
    SELECT es."version"
    INTO current_event_stream_version
    FROM event_streams es
    WHERE es.event_stream_id = _event_stream_id;

    IF NOT FOUND THEN
        current_event_stream_version := 0;
    END IF;

    new_event_stream_version := current_event_stream_version + _new_version_offset;

    INSERT INTO event_streams (event_stream_id, "version")
    VALUES (_event_stream_id, new_event_stream_version)
    ON CONFLICT (event_stream_id) DO
    UPDATE SET "version" = new_event_stream_version;

    RETURN new_event_stream_version;
END;
$$;
