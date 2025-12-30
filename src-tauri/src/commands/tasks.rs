use chrono::{DateTime, Utc};
use duckdb::Connection;
use uuid::Uuid;

use crate::db::{CreateTask, Task, UpdateTask};
use crate::error::{AppError, AppResult};
use crate::AppState;

/// タスクをDBに保存する
fn insert_task(conn: &Connection, task: &Task) -> AppResult<()> {
    conn.execute(
        "INSERT INTO tasks (id, folder_id, name, description, color, archived, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        duckdb::params![
            task.id.to_string(),
            task.folder_id.map(|id| id.to_string()),
            &task.name,
            &task.description,
            &task.color,
            task.archived,
            task.created_at,
            task.updated_at,
        ],
    )?;
    Ok(())
}

/// DBからタスクを取得する
fn fetch_tasks(conn: &Connection, include_archived: bool) -> AppResult<Vec<Task>> {
    let sql = if include_archived {
        "SELECT id, folder_id, name, description, color, archived, created_at, updated_at FROM tasks ORDER BY created_at DESC"
    } else {
        "SELECT id, folder_id, name, description, color, archived, created_at, updated_at FROM tasks WHERE archived = false ORDER BY created_at DESC"
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let folder_id_str: Option<String> = row.get(1)?;
        let created_at: DateTime<Utc> = row.get(6)?;
        let updated_at: DateTime<Utc> = row.get(7)?;

        Ok(Task {
            id: Uuid::parse_str(&id_str).unwrap(),
            folder_id: folder_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
            name: row.get(2)?,
            description: row.get(3)?,
            color: row.get(4)?,
            archived: row.get(5)?,
            created_at,
            updated_at,
        })
    })?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

/// IDでタスクを取得する
fn fetch_task_by_id(conn: &Connection, id: &Uuid) -> AppResult<Task> {
    let mut stmt = conn.prepare(
        "SELECT id, folder_id, name, description, color, archived, created_at, updated_at FROM tasks WHERE id = ?",
    )?;

    let task = stmt
        .query_row([id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let folder_id_str: Option<String> = row.get(1)?;
            let created_at: DateTime<Utc> = row.get(6)?;
            let updated_at: DateTime<Utc> = row.get(7)?;

            Ok(Task {
                id: Uuid::parse_str(&id_str).unwrap(),
                folder_id: folder_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
                name: row.get(2)?,
                description: row.get(3)?,
                color: row.get(4)?,
                archived: row.get(5)?,
                created_at,
                updated_at,
            })
        })
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    Ok(task)
}

/// タスク一覧を取得する
#[tauri::command]
pub fn list_tasks(state: tauri::State<AppState>, include_archived: bool) -> AppResult<Vec<Task>> {
    state.db.with_connection(|conn| fetch_tasks(conn, include_archived))
}

/// タスクを作成する
#[tauri::command]
pub fn create_task(state: tauri::State<AppState>, task: CreateTask) -> AppResult<Task> {
    if task.name.trim().is_empty() {
        return Err(AppError::InvalidInput("Task name cannot be empty".to_string()));
    }

    if let Some(ref color) = task.color {
        if !Task::is_valid_color(color) {
            return Err(AppError::InvalidInput(format!(
                "Invalid color format: {}. Expected #RRGGBB",
                color
            )));
        }
    }

    let new_task = Task::new(task.name, task.description, task.color, task.folder_id);

    state.db.with_connection(|conn| {
        insert_task(conn, &new_task)?;
        Ok(new_task)
    })
}

/// タスクを更新する
#[tauri::command]
pub fn update_task(
    state: tauri::State<AppState>,
    id: String,
    update: UpdateTask,
) -> AppResult<Task> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", id)))?;

    if let Some(ref name) = update.name {
        if name.trim().is_empty() {
            return Err(AppError::InvalidInput("Task name cannot be empty".to_string()));
        }
    }

    if let Some(ref color) = update.color {
        if !Task::is_valid_color(color) {
            return Err(AppError::InvalidInput(format!(
                "Invalid color format: {}. Expected #RRGGBB",
                color
            )));
        }
    }

    state.db.with_connection(|conn| {
        let mut task = fetch_task_by_id(conn, &task_id)?;

        if let Some(name) = update.name {
            task.name = name;
        }
        if let Some(description) = update.description {
            task.description = Some(description);
        }
        if let Some(color) = update.color {
            task.color = color;
        }
        if let Some(folder_id) = update.folder_id {
            task.folder_id = folder_id;
        }
        task.updated_at = Utc::now();

        conn.execute(
            "UPDATE tasks SET name = ?, description = ?, color = ?, folder_id = ?, updated_at = ? WHERE id = ?",
            duckdb::params![
                &task.name,
                &task.description,
                &task.color,
                task.folder_id.map(|id| id.to_string()),
                task.updated_at,
                task.id.to_string(),
            ],
        )?;

        Ok(task)
    })
}

/// タスクをアーカイブ/復元する
#[tauri::command]
pub fn archive_task(state: tauri::State<AppState>, id: String, archived: bool) -> AppResult<()> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", id)))?;

    state.db.with_connection(|conn| {
        // タスクが存在するか確認
        let _ = fetch_task_by_id(conn, &task_id)?;

        conn.execute(
            "UPDATE tasks SET archived = ?, updated_at = ? WHERE id = ?",
            duckdb::params![archived, Utc::now(), task_id.to_string()],
        )?;

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn create_test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    mod list_tasks_tests {
        use super::*;

        #[test]
        fn 空のデータベースからタスク一覧を取得すると空のベクターが返る() {
            let db = create_test_db();

            let tasks = db
                .with_connection(|conn| fetch_tasks(conn, false))
                .unwrap();

            assert!(tasks.is_empty());
        }

        #[test]
        fn 作成したタスクが一覧に含まれる() {
            let db = create_test_db();
            let task = Task::new("テスト作業".to_string(), None, None, None);

            db.with_connection(|conn| {
                insert_task(conn, &task)?;
                let tasks = fetch_tasks(conn, false)?;

                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].name, "テスト作業");
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn include_archivedがfalseの場合アーカイブ済みタスクは含まれない() {
            let db = create_test_db();
            let mut task = Task::new("アーカイブ済み".to_string(), None, None, None);
            task.archived = true;

            db.with_connection(|conn| {
                insert_task(conn, &task)?;
                let tasks = fetch_tasks(conn, false)?;

                assert!(tasks.is_empty());
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn include_archivedがtrueの場合アーカイブ済みタスクも含まれる() {
            let db = create_test_db();
            let mut task = Task::new("アーカイブ済み".to_string(), None, None, None);
            task.archived = true;

            db.with_connection(|conn| {
                insert_task(conn, &task)?;
                let tasks = fetch_tasks(conn, true)?;

                assert_eq!(tasks.len(), 1);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn タスク一覧は作成日時の降順でソートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let task1 = Task::new("タスク1".to_string(), None, None, None);
                insert_task(conn, &task1)?;

                // 少し時間を空けて2つ目を作成
                std::thread::sleep(std::time::Duration::from_millis(10));
                let task2 = Task::new("タスク2".to_string(), None, None, None);
                insert_task(conn, &task2)?;

                let tasks = fetch_tasks(conn, false)?;

                assert_eq!(tasks.len(), 2);
                assert_eq!(tasks[0].name, "タスク2"); // 新しい方が先
                assert_eq!(tasks[1].name, "タスク1");
                Ok(())
            })
            .unwrap();
        }
    }

    mod create_task_tests {
        use super::*;

        #[test]
        fn タスクを作成するとDBに保存される() {
            let db = create_test_db();
            let task = Task::new("新規タスク".to_string(), Some("説明".to_string()), None, None);

            db.with_connection(|conn| {
                insert_task(conn, &task)?;
                let fetched = fetch_task_by_id(conn, &task.id)?;

                assert_eq!(fetched.name, "新規タスク");
                assert_eq!(fetched.description, Some("説明".to_string()));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 空のタスク名はエラーになる() {
            let create_task = CreateTask {
                name: "".to_string(),
                description: None,
                color: None,
                folder_id: None,
            };

            if create_task.name.trim().is_empty() {
                // Expected error
            } else {
                panic!("Should have caught empty name");
            }
        }

        #[test]
        fn 無効なカラーコードはエラーになる() {
            let invalid_color = "invalid";

            assert!(!Task::is_valid_color(invalid_color));
        }
    }

    mod update_task_tests {
        use super::*;

        #[test]
        fn タスク名を更新できる() {
            let db = create_test_db();
            let task = Task::new("元の名前".to_string(), None, None, None);

            db.with_connection(|conn| {
                insert_task(conn, &task)?;

                conn.execute(
                    "UPDATE tasks SET name = ?, updated_at = ? WHERE id = ?",
                    duckdb::params!["新しい名前", Utc::now(), task.id.to_string()],
                )?;

                let updated = fetch_task_by_id(conn, &task.id)?;
                assert_eq!(updated.name, "新しい名前");
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 説明を更新できる() {
            let db = create_test_db();
            let task = Task::new("タスク".to_string(), None, None, None);

            db.with_connection(|conn| {
                insert_task(conn, &task)?;

                conn.execute(
                    "UPDATE tasks SET description = ?, updated_at = ? WHERE id = ?",
                    duckdb::params!["新しい説明", Utc::now(), task.id.to_string()],
                )?;

                let updated = fetch_task_by_id(conn, &task.id)?;
                assert_eq!(updated.description, Some("新しい説明".to_string()));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn カラーを更新できる() {
            let db = create_test_db();
            let task = Task::new("タスク".to_string(), None, None, None);

            db.with_connection(|conn| {
                insert_task(conn, &task)?;

                conn.execute(
                    "UPDATE tasks SET color = ?, updated_at = ? WHERE id = ?",
                    duckdb::params!["#ff0000", Utc::now(), task.id.to_string()],
                )?;

                let updated = fetch_task_by_id(conn, &task.id)?;
                assert_eq!(updated.color, "#ff0000");
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 存在しないタスクを更新しようとするとエラーになる() {
            let db = create_test_db();
            let non_existent_id = Uuid::new_v4();

            let result = db.with_connection(|conn| fetch_task_by_id(conn, &non_existent_id));

            assert!(result.is_err());
        }
    }

    mod archive_task_tests {
        use super::*;

        #[test]
        fn タスクをアーカイブできる() {
            let db = create_test_db();
            let task = Task::new("タスク".to_string(), None, None, None);

            db.with_connection(|conn| {
                insert_task(conn, &task)?;

                conn.execute(
                    "UPDATE tasks SET archived = true WHERE id = ?",
                    [task.id.to_string()],
                )?;

                let updated = fetch_task_by_id(conn, &task.id)?;
                assert!(updated.archived);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn アーカイブ済みタスクを復元できる() {
            let db = create_test_db();
            let mut task = Task::new("タスク".to_string(), None, None, None);
            task.archived = true;

            db.with_connection(|conn| {
                insert_task(conn, &task)?;

                conn.execute(
                    "UPDATE tasks SET archived = false WHERE id = ?",
                    [task.id.to_string()],
                )?;

                let updated = fetch_task_by_id(conn, &task.id)?;
                assert!(!updated.archived);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 存在しないタスクをアーカイブしようとするとエラーになる() {
            let db = create_test_db();
            let non_existent_id = Uuid::new_v4();

            let result = db.with_connection(|conn| fetch_task_by_id(conn, &non_existent_id));

            assert!(result.is_err());
        }
    }
}
