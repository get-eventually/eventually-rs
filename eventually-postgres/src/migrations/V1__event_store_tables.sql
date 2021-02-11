CREATE TABLE aggregate_types (
    id       TEXT   PRIMARY KEY,
    "offset" BIGINT NOT NULL    DEFAULT -1
);

CREATE TABLE aggregates (
    id                TEXT    NOT NULL,
    aggregate_type_id TEXT    NOT NULL,
    "version"         INTEGER NOT NULL    DEFAULT 0,

    PRIMARY KEY (id, aggregate_type_id),
    -- Remove all aggregates in case the aggregate type is deleted.
    FOREIGN KEY (aggregate_type_id) REFERENCES aggregate_types(id) ON DELETE CASCADE
);

CREATE SEQUENCE events_number_seq AS BIGINT MINVALUE 0;
CREATE TABLE events (
    aggregate_id    TEXT    NOT NULL,
    aggregate_type  TEXT    NOT NULL,
    "version"       INTEGER NOT NULL,
    sequence_number BIGINT  NOT NULL    DEFAULT nextval('events_number_seq'),
    "event"         JSONB   NOT NULL,

    CONSTRAINT unique_sequence UNIQUE(sequence_number),
    PRIMARY KEY (aggregate_id, aggregate_type, "version"),
    -- Remove all the events of the aggregate in case of delete.
    FOREIGN KEY (aggregate_id, aggregate_type) REFERENCES aggregates(id, aggregate_type_id) ON DELETE CASCADE
);
ALTER SEQUENCE events_number_seq OWNED BY events.sequence_number;
