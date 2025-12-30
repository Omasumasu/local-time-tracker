use std::path::Path;
use std::sync::Mutex;

use duckdb::Connection;

use crate::error::{AppError, AppResult};

use super::migrations::run_migrations;

/// データベース管理構造体
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// メモリ上にデータベースを作成する（テスト用）
    pub fn new_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// ファイルベースのデータベースを開く/作成する
    pub fn open<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// データベース接続を取得してクロージャを実行する
    pub fn with_connection<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::OperationFailed(format!("Failed to acquire lock: {}", e)))?;
        f(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn メモリ上にデータベースを作成できる() {
        let db = Database::new_in_memory();

        assert!(db.is_ok());
    }

    #[test]
    fn データベース作成時にマイグレーションが自動実行される() {
        let db = Database::new_in_memory().unwrap();

        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'tasks'",
            )?;
            let count: i64 = stmt.query_row([], |row| row.get(0))?;
            Ok(count)
        });

        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn with_connectionでクエリを実行できる() {
        let db = Database::new_in_memory().unwrap();

        let result = db.with_connection(|conn| {
            conn.execute("SELECT 1", [])?;
            Ok(())
        });

        assert!(result.is_ok());
    }

    #[test]
    fn ファイルベースのデータベースを作成できる() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(&db_path);

        assert!(db.is_ok());
        assert!(db_path.exists());
    }

    #[test]
    fn ファイルベースのデータベースを再度開くと既存のテーブルが利用できる() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // 最初に作成
        {
            let db = Database::open(&db_path).unwrap();
            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at) VALUES (uuid(), 'テスト', '#000000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        }

        // 再度開く
        {
            let db = Database::open(&db_path).unwrap();
            let count = db
                .with_connection(|conn| {
                    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tasks")?;
                    let count: i64 = stmt.query_row([], |row| row.get(0))?;
                    Ok(count)
                })
                .unwrap();

            assert_eq!(count, 1);
        }
    }
}
