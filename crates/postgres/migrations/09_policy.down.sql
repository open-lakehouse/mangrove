-- Postgres cannot remove a value from an enum type without recreating the type
-- and rewriting every dependent column, which is unsafe to do automatically.
-- The added `policy_info` value is therefore left in place on rollback.
SELECT 1;
