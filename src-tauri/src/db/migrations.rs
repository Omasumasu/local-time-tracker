use duckdb::Connection;

use crate::error::AppResult;

const MIGRATION_SQL: &str = include_str!("../../migrations/001_initial.sql");

/// マイグレーションを実行する
pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(MIGRATION_SQL)?;
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
