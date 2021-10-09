CREATE OR REPLACE FUNCTION get_or_create_subscription(
    name              TEXT,
    aggregate_type_id TEXT
)
RETURNS subscriptions
AS $$

    INSERT INTO subscriptions (name, aggregate_type_id)
    VALUES (name, aggregate_type_id)
        ON CONFLICT (name)
        DO UPDATE SET name=EXCLUDED.name -- Perform update to force row returning.
    RETURNING *;

$$ LANGUAGE SQL;
