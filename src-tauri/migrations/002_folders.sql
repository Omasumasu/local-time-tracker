-- Migration: Add folders support

-- Create folders table if not exists
CREATE TABLE IF NOT EXISTS folders (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    color VARCHAR(7) NOT NULL DEFAULT '#6b7280',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Add folder_id column to tasks if not exists
-- DuckDB doesn't support IF NOT EXISTS for ALTER TABLE, so we use a workaround
DO $$
BEGIN
    ALTER TABLE tasks ADD COLUMN folder_id VARCHAR;
EXCEPTION
    WHEN duplicate_column THEN NULL;
END $$;

-- Create index on folders.sort_order if not exists
CREATE INDEX IF NOT EXISTS idx_folders_sort_order ON folders(sort_order);
