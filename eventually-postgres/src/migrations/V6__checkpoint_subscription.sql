CREATE OR REPLACE FUNCTION checkpoint_subscription(
    subscription_name TEXT,
    aggregate_type    TEXT,
    sequence_number   BIGINT
)
RETURNS VOID
AS $$
DECLARE
    old_sequence_number BIGINT;
BEGIN

    SELECT s.last_sequence_number INTO old_sequence_number
    FROM subscriptions s
    WHERE s."name" = subscription_name AND s.aggregate_type_id = aggregate_type;

    IF old_sequence_number >= sequence_number THEN
        RAISE EXCEPTION 'invalid sequence number provided: %, should be greater than %', sequence_number, old_sequence_number;
    END IF;

    UPDATE subscriptions s
    SET last_sequence_number = sequence_number, updated_at = NOW()
    WHERE s."name" = subscription_name AND s.aggregate_type_id = aggregate_type;

END;
$$ LANGUAGE PLPGSQL;
