-- Tasks table: 作業内容
CREATE TABLE IF NOT EXISTS tasks (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    color VARCHAR(7) NOT NULL DEFAULT '#3b82f6',
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Artifacts table: 成果物
CREATE TABLE IF NOT EXISTS artifacts (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    artifact_type VARCHAR(50) NOT NULL,
    reference TEXT,
    metadata JSON,
    created_at TIMESTAMPTZ NOT NULL
);

-- Time entries table: 作業記録
CREATE TABLE IF NOT EXISTS time_entries (
    id VARCHAR PRIMARY KEY,
    task_id VARCHAR,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Entry artifacts table: 紐付けテーブル
CREATE TABLE IF NOT EXISTS entry_artifacts (
    entry_id VARCHAR NOT NULL,
    artifact_id VARCHAR NOT NULL,
    PRIMARY KEY (entry_id, artifact_id)
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_time_entries_task_id ON time_entries(task_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_started_at ON time_entries(started_at);
CREATE INDEX IF NOT EXISTS idx_time_entries_ended_at ON time_entries(ended_at);
CREATE INDEX IF NOT EXISTS idx_tasks_archived ON tasks(archived);
