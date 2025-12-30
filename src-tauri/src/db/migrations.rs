use duckdb::Connection;

use crate::error::AppResult;

const MIGRATION_SQL: &str = include_str!("../../migrations/001_initial.sql");

/// マイグレーションを実行する
pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    // Run base migrations (creates tables if they don't exist)
    conn.execute_batch(MIGRATION_SQL)?;

    // Schema upgrade: Add folder_id column to tasks if it doesn't exist
    // (for databases created before folder feature was added)
    let has_folder_id: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM information_schema.columns
             WHERE table_name = 'tasks' AND column_name = 'folder_id'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_folder_id {
        conn.execute_batch("ALTER TABLE tasks ADD COLUMN folder_id VARCHAR")?;
    }

    // Schema upgrade: Add icon column to folders if it doesn't exist
    let has_icon: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM information_schema.columns
             WHERE table_name = 'folders' AND column_name = 'icon'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_icon {
        conn.execute_batch("ALTER TABLE folders ADD COLUMN icon VARCHAR(50)")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn マイグレーションを実行するとテーブルが作成される() {
        let conn = Connection::open_in_memory().unwrap();
        let result = run_migrations(&conn);

        assert!(result.is_ok());
    }

    #[test]
    fn マイグレーション後にtasksテーブルが存在する() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'tasks'")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn マイグレーション後にartifactsテーブルが存在する() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'artifacts'",
            )
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn マイグレーション後にtime_entriesテーブルが存在する() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'time_entries'",
            )
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn マイグレーション後にentry_artifactsテーブルが存在する() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'entry_artifacts'",
            )
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn マイグレーションは冪等性がある_複数回実行しても問題ない() {
        let conn = Connection::open_in_memory().unwrap();

        let result1 = run_migrations(&conn);
        let result2 = run_migrations(&conn);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
