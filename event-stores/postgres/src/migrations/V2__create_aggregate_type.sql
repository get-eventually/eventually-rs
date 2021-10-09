CREATE OR REPLACE FUNCTION create_aggregate_type(aggregate_type TEXT)
RETURNS VOID
AS $$

    INSERT INTO aggregate_types (id) VALUES (aggregate_type)
    ON CONFLICT (id)
    DO NOTHING;

$$ LANGUAGE SQL;
