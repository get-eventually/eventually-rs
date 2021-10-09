CREATE TABLE subscriptions (
    name                 TEXT        PRIMARY KEY,
    aggregate_type_id    TEXT        NOT NULL,
    last_sequence_number BIGINT      NOT NULL DEFAULT -1,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Remove all subscriptions in case the aggregate type is deleted.
    FOREIGN KEY (aggregate_type_id) REFERENCES aggregate_types(id) ON DELETE CASCADE
);
