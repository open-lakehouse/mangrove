-- no-transaction
-- Add the `policy_info` object label for ABAC row-filter / column-mask policies.
--
-- `ALTER TYPE ... ADD VALUE` cannot run inside a transaction block, hence the
-- `-- no-transaction` directive above (honored by sqlx's migrator).
--
-- Policies are read as whole documents, never traversed, so they are stored as plain
-- objects (properties JSONB) — no new association labels are introduced here.
ALTER TYPE object_label ADD VALUE IF NOT EXISTS 'policy_info';
