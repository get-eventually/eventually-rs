CREATE TABLE aggregate_types (
    id       TEXT   PRIMARY KEY,
    "offset" BIGINT NOT NULL    DEFAULT -1
);

CREATE TABLE aggregates (
    id                TEXT    PRIMARY KEY,
    aggregate_type_id TEXT    NOT NULL,
    "version"         INTEGER NOT NULL    DEFAULT 0,

    -- Remove all aggregates in case the aggregate type is deleted.
    FOREIGN KEY (aggregate_type_id) REFERENCES aggregate_types(id) ON DELETE CASCADE
);

CREATE TABLE events (
    aggregate_id    TEXT    NOT NULL,
    "version"       INTEGER NOT NULL,
    sequence_number BIGINT  NOT NULL,
    "event"         JSONB   NOT NULL,

    PRIMARY KEY (aggregate_id, "version"),
    -- Remove all the events of the aggregate in case of delete.
    FOREIGN KEY (aggregate_id) REFERENCES aggregates(id) ON DELETE CASCADE
);
