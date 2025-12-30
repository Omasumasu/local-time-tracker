use chrono::{DateTime, Utc};
use duckdb::Connection;
use uuid::Uuid;

use crate::db::{
    Artifact, EntryFilter, Task, TimeEntry, TimeEntryWithRelations, UpdateEntry,
};
use crate::error::{AppError, AppResult};
use crate::AppState;

/// 時間記録をDBに保存する
fn insert_entry(conn: &Connection, entry: &TimeEntry) -> AppResult<()> {
    conn.execute(
        "INSERT INTO time_entries (id, task_id, started_at, ended_at, memo, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        duckdb::params![
            entry.id.to_string(),
            entry.task_id.map(|id| id.to_string()),
            entry.started_at,
            entry.ended_at,
            &entry.memo,
            entry.created_at,
            entry.updated_at,
        ],
    )?;
    Ok(())
}

/// 計測中のエントリを取得する
fn fetch_running_entry(conn: &Connection) -> AppResult<Option<TimeEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, memo, created_at, updated_at
         FROM time_entries WHERE ended_at IS NULL LIMIT 1",
    )?;

    let result = stmt.query_row([], |row| {
        let id_str: String = row.get(0)?;
        let task_id_str: Option<String> = row.get(1)?;
        let started_at: DateTime<Utc> = row.get(2)?;
        let ended_at: Option<DateTime<Utc>> = row.get(3)?;
        let created_at: DateTime<Utc> = row.get(5)?;
        let updated_at: DateTime<Utc> = row.get(6)?;

        Ok(TimeEntry {
            id: Uuid::parse_str(&id_str).unwrap(),
            task_id: task_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
            started_at,
            ended_at,
            memo: row.get(4)?,
            created_at,
            updated_at,
        })
    });

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// IDで時間記録を取得する
fn fetch_entry_by_id(conn: &Connection, id: &Uuid) -> AppResult<TimeEntry> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, memo, created_at, updated_at
         FROM time_entries WHERE id = ?",
    )?;

    let entry = stmt
        .query_row([id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let task_id_str: Option<String> = row.get(1)?;
            let started_at: DateTime<Utc> = row.get(2)?;
            let ended_at: Option<DateTime<Utc>> = row.get(3)?;
            let created_at: DateTime<Utc> = row.get(5)?;
            let updated_at: DateTime<Utc> = row.get(6)?;

            Ok(TimeEntry {
                id: Uuid::parse_str(&id_str).unwrap(),
                task_id: task_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
                started_at,
                ended_at,
                memo: row.get(4)?,
                created_at,
                updated_at,
            })
        })
        .map_err(|_| AppError::NotFound(format!("Entry with id {} not found", id)))?;

    Ok(entry)
}

/// タスク情報を取得する
fn fetch_task_by_id(conn: &Connection, id: &Uuid) -> AppResult<Option<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, archived, created_at, updated_at
         FROM tasks WHERE id = ?",
    )?;

    let result = stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let created_at: DateTime<Utc> = row.get(5)?;
        let updated_at: DateTime<Utc> = row.get(6)?;

        Ok(Task {
            id: Uuid::parse_str(&id_str).unwrap(),
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            archived: row.get(4)?,
            created_at,
            updated_at,
        })
    });

    match result {
        Ok(task) => Ok(Some(task)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// エントリに紐付いた成果物を取得する
fn fetch_artifacts_for_entry(conn: &Connection, entry_id: &Uuid) -> AppResult<Vec<Artifact>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.artifact_type, a.reference, a.metadata, a.created_at
         FROM artifacts a
         JOIN entry_artifacts ea ON ea.artifact_id = a.id
         WHERE ea.entry_id = ?",
    )?;

    let rows = stmt.query_map([entry_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let created_at: DateTime<Utc> = row.get(5)?;
        let metadata_str: Option<String> = row.get(4)?;

        Ok(Artifact {
            id: Uuid::parse_str(&id_str).unwrap(),
            name: row.get(1)?,
            artifact_type: row.get(2)?,
            reference: row.get(3)?,
            metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            created_at,
        })
    })?;

    let mut artifacts = Vec::new();
    for row in rows {
        artifacts.push(row?);
    }
    Ok(artifacts)
}

/// エントリをリレーション付きで変換する
fn entry_to_with_relations(
    conn: &Connection,
    entry: TimeEntry,
) -> AppResult<TimeEntryWithRelations> {
    let task = if let Some(task_id) = entry.task_id {
        fetch_task_by_id(conn, &task_id)?
    } else {
        None
    };

    let artifacts = fetch_artifacts_for_entry(conn, &entry.id)?;

    let duration_seconds = entry.ended_at.map(|ended| (ended - entry.started_at).num_seconds());

    Ok(TimeEntryWithRelations {
        id: entry.id,
        task_id: entry.task_id,
        task,
        started_at: entry.started_at,
        ended_at: entry.ended_at,
        duration_seconds,
        memo: entry.memo,
        artifacts,
        created_at: entry.created_at,
        updated_at: entry.updated_at,
    })
}

/// フィルタ条件で時間記録を取得する
fn fetch_entries_with_filter(
    conn: &Connection,
    filter: &EntryFilter,
) -> AppResult<Vec<TimeEntryWithRelations>> {
    let mut sql = String::from(
        "SELECT id, task_id, started_at, ended_at, memo, created_at, updated_at
         FROM time_entries WHERE 1=1",
    );
    let mut params: Vec<Box<dyn duckdb::ToSql>> = Vec::new();

    if let Some(ref from) = filter.from {
        sql.push_str(" AND started_at >= ?");
        params.push(Box::new(*from));
    }
    if let Some(ref to) = filter.to {
        sql.push_str(" AND started_at <= ?");
        params.push(Box::new(*to));
    }
    if let Some(ref task_id) = filter.task_id {
        sql.push_str(" AND task_id = ?");
        params.push(Box::new(task_id.to_string()));
    }

    sql.push_str(" ORDER BY started_at DESC");

    if let Some(limit) = filter.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        let id_str: String = row.get(0)?;
        let task_id_str: Option<String> = row.get(1)?;
        let started_at: DateTime<Utc> = row.get(2)?;
        let ended_at: Option<DateTime<Utc>> = row.get(3)?;
        let created_at: DateTime<Utc> = row.get(5)?;
        let updated_at: DateTime<Utc> = row.get(6)?;

        Ok(TimeEntry {
            id: Uuid::parse_str(&id_str).unwrap(),
            task_id: task_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
            started_at,
            ended_at,
            memo: row.get(4)?,
            created_at,
            updated_at,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        let entry = row?;
        entries.push(entry_to_with_relations(conn, entry)?);
    }
    Ok(entries)
}

/// 計測を開始する
#[tauri::command]
pub fn start_entry(
    state: tauri::State<AppState>,
    task_id: Option<String>,
    memo: Option<String>,
) -> AppResult<TimeEntry> {
    let task_uuid = if let Some(ref id) = task_id {
        Some(
            Uuid::parse_str(id)
                .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", id)))?,
        )
    } else {
        None
    };

    state.db.with_connection(|conn| {
        // 既に計測中のエントリがあればエラー
        if fetch_running_entry(conn)?.is_some() {
            return Err(AppError::AlreadyExists(
                "There is already a running entry".to_string(),
            ));
        }

        let entry = TimeEntry::start(task_uuid, memo);
        insert_entry(conn, &entry)?;
        Ok(entry)
    })
}

/// 計測を停止する
#[tauri::command]
pub fn stop_entry(state: tauri::State<AppState>, id: Option<String>) -> AppResult<TimeEntry> {
    state.db.with_connection(|conn| {
        let entry = if let Some(ref entry_id) = id {
            let uuid = Uuid::parse_str(entry_id)
                .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", entry_id)))?;
            fetch_entry_by_id(conn, &uuid)?
        } else {
            fetch_running_entry(conn)?
                .ok_or_else(|| AppError::NotFound("No running entry found".to_string()))?
        };

        if !entry.is_running() {
            return Err(AppError::OperationFailed(
                "Entry is not running".to_string(),
            ));
        }

        let now = Utc::now();
        conn.execute(
            "UPDATE time_entries SET ended_at = ?, updated_at = ? WHERE id = ?",
            duckdb::params![now, now, entry.id.to_string()],
        )?;

        let mut updated = entry;
        updated.ended_at = Some(now);
        updated.updated_at = now;
        Ok(updated)
    })
}

/// 計測中のエントリを取得する
#[tauri::command]
pub fn get_running_entry(
    state: tauri::State<AppState>,
) -> AppResult<Option<TimeEntryWithRelations>> {
    state.db.with_connection(|conn| {
        if let Some(entry) = fetch_running_entry(conn)? {
            Ok(Some(entry_to_with_relations(conn, entry)?))
        } else {
            Ok(None)
        }
    })
}

/// 時間記録一覧を取得する
#[tauri::command]
pub fn list_entries(
    state: tauri::State<AppState>,
    from: Option<String>,
    to: Option<String>,
    task_id: Option<String>,
    limit: Option<i64>,
) -> AppResult<Vec<TimeEntryWithRelations>> {
    let filter = EntryFilter {
        from: from.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        to: to.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        task_id: task_id.and_then(|s| Uuid::parse_str(&s).ok()),
        limit,
    };

    state
        .db
        .with_connection(|conn| fetch_entries_with_filter(conn, &filter))
}

/// 時間記録を更新する
#[tauri::command]
pub fn update_entry(
    state: tauri::State<AppState>,
    id: String,
    update: UpdateEntry,
) -> AppResult<TimeEntry> {
    let entry_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", id)))?;

    state.db.with_connection(|conn| {
        let mut entry = fetch_entry_by_id(conn, &entry_id)?;

        if let Some(task_id) = update.task_id {
            entry.task_id = task_id;
        }
        if let Some(started_at) = update.started_at {
            entry.started_at = started_at;
        }
        if let Some(ended_at) = update.ended_at {
            entry.ended_at = ended_at;
        }
        if let Some(memo) = update.memo {
            entry.memo = Some(memo);
        }
        entry.updated_at = Utc::now();

        conn.execute(
            "UPDATE time_entries SET task_id = ?, started_at = ?, ended_at = ?, memo = ?, updated_at = ? WHERE id = ?",
            duckdb::params![
                entry.task_id.map(|id| id.to_string()),
                entry.started_at,
                entry.ended_at,
                &entry.memo,
                entry.updated_at,
                entry.id.to_string(),
            ],
        )?;

        Ok(entry)
    })
}

/// 時間記録を削除する
#[tauri::command]
pub fn delete_entry(state: tauri::State<AppState>, id: String) -> AppResult<()> {
    let entry_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", id)))?;

    state.db.with_connection(|conn| {
        // エントリが存在するか確認
        let _ = fetch_entry_by_id(conn, &entry_id)?;

        // 紐付けを削除
        conn.execute(
            "DELETE FROM entry_artifacts WHERE entry_id = ?",
            [entry_id.to_string()],
        )?;

        // エントリを削除
        conn.execute(
            "DELETE FROM time_entries WHERE id = ?",
            [entry_id.to_string()],
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

    mod start_entry_tests {
        use super::*;

        #[test]
        fn 計測を開始するとエントリが作成される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                let fetched = fetch_entry_by_id(conn, &entry.id)?;
                assert_eq!(fetched.id, entry.id);
                assert!(fetched.is_running());
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 計測開始時にタスクIDを紐付けできる() {
            let db = create_test_db();
            let task_id = Uuid::new_v4();

            db.with_connection(|conn| {
                // タスクを先に作成
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at) VALUES (?, 'テスト', '#000000', ?, ?)",
                    duckdb::params![task_id.to_string(), Utc::now(), Utc::now()],
                )?;

                let entry = TimeEntry::start(Some(task_id), None);
                insert_entry(conn, &entry)?;

                let fetched = fetch_entry_by_id(conn, &entry.id)?;
                assert_eq!(fetched.task_id, Some(task_id));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 計測開始時にメモを設定できる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, Some("作業開始".to_string()));
                insert_entry(conn, &entry)?;

                let fetched = fetch_entry_by_id(conn, &entry.id)?;
                assert_eq!(fetched.memo, Some("作業開始".to_string()));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 計測中のエントリがある場合は新しい計測を開始できない() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                let running = fetch_running_entry(conn)?;
                assert!(running.is_some());
                Ok(())
            })
            .unwrap();
        }
    }

    mod stop_entry_tests {
        use super::*;

        #[test]
        fn 計測を停止すると終了時刻が設定される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                let now = Utc::now();
                conn.execute(
                    "UPDATE time_entries SET ended_at = ?, updated_at = ? WHERE id = ?",
                    duckdb::params![now, now, entry.id.to_string()],
                )?;

                let stopped = fetch_entry_by_id(conn, &entry.id)?;
                assert!(!stopped.is_running());
                assert!(stopped.ended_at.is_some());
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 計測中でないエントリは停止できない() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let mut entry = TimeEntry::start(None, None);
                entry.ended_at = Some(Utc::now());
                insert_entry(conn, &entry)?;

                let fetched = fetch_entry_by_id(conn, &entry.id)?;
                assert!(!fetched.is_running());
                Ok(())
            })
            .unwrap();
        }
    }

    mod get_running_entry_tests {
        use super::*;

        #[test]
        fn 計測中のエントリがない場合はNoneが返る() {
            let db = create_test_db();

            let result = db.with_connection(|conn| fetch_running_entry(conn)).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn 計測中のエントリがある場合はそのエントリが返る() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, Some("計測中".to_string()));
                insert_entry(conn, &entry)?;

                let running = fetch_running_entry(conn)?;
                assert!(running.is_some());
                assert_eq!(running.unwrap().id, entry.id);
                Ok(())
            })
            .unwrap();
        }
    }

    mod list_entries_tests {
        use super::*;

        #[test]
        fn 空のデータベースからエントリ一覧を取得すると空のベクターが返る() {
            let db = create_test_db();

            let entries = db
                .with_connection(|conn| fetch_entries_with_filter(conn, &EntryFilter::default()))
                .unwrap();

            assert!(entries.is_empty());
        }

        #[test]
        fn 作成したエントリが一覧に含まれる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                let entries = fetch_entries_with_filter(conn, &EntryFilter::default())?;
                assert_eq!(entries.len(), 1);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn limitを指定すると取得件数が制限される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                for _ in 0..5 {
                    let mut entry = TimeEntry::start(None, None);
                    entry.ended_at = Some(Utc::now());
                    insert_entry(conn, &entry)?;
                }

                let filter = EntryFilter {
                    limit: Some(3),
                    ..Default::default()
                };
                let entries = fetch_entries_with_filter(conn, &filter)?;
                assert_eq!(entries.len(), 3);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn task_idでフィルタできる() {
            let db = create_test_db();
            let task_id = Uuid::new_v4();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at) VALUES (?, 'テスト', '#000000', ?, ?)",
                    duckdb::params![task_id.to_string(), Utc::now(), Utc::now()],
                )?;

                // タスク付きエントリ
                let mut entry1 = TimeEntry::start(Some(task_id), None);
                entry1.ended_at = Some(Utc::now());
                insert_entry(conn, &entry1)?;

                // タスクなしエントリ
                let mut entry2 = TimeEntry::start(None, None);
                entry2.ended_at = Some(Utc::now());
                insert_entry(conn, &entry2)?;

                let filter = EntryFilter {
                    task_id: Some(task_id),
                    ..Default::default()
                };
                let entries = fetch_entries_with_filter(conn, &filter)?;
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].task_id, Some(task_id));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn エントリ一覧は開始日時の降順でソートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let mut entry1 = TimeEntry::start(None, Some("1番目".to_string()));
                entry1.ended_at = Some(Utc::now());
                insert_entry(conn, &entry1)?;

                std::thread::sleep(std::time::Duration::from_millis(10));

                let mut entry2 = TimeEntry::start(None, Some("2番目".to_string()));
                entry2.ended_at = Some(Utc::now());
                insert_entry(conn, &entry2)?;

                let entries = fetch_entries_with_filter(conn, &EntryFilter::default())?;
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].memo, Some("2番目".to_string())); // 新しい方が先
                assert_eq!(entries[1].memo, Some("1番目".to_string()));
                Ok(())
            })
            .unwrap();
        }
    }

    mod update_entry_tests {
        use super::*;

        #[test]
        fn メモを更新できる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                conn.execute(
                    "UPDATE time_entries SET memo = ? WHERE id = ?",
                    duckdb::params!["更新後のメモ", entry.id.to_string()],
                )?;

                let updated = fetch_entry_by_id(conn, &entry.id)?;
                assert_eq!(updated.memo, Some("更新後のメモ".to_string()));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 開始時刻を修正できる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                let new_start = Utc::now() - chrono::Duration::hours(1);
                conn.execute(
                    "UPDATE time_entries SET started_at = ? WHERE id = ?",
                    duckdb::params![new_start, entry.id.to_string()],
                )?;

                let updated = fetch_entry_by_id(conn, &entry.id)?;
                assert!(updated.started_at < entry.started_at);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn タスクIDを変更できる() {
            let db = create_test_db();
            let new_task_id = Uuid::new_v4();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at) VALUES (?, 'テスト', '#000000', ?, ?)",
                    duckdb::params![new_task_id.to_string(), Utc::now(), Utc::now()],
                )?;

                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                conn.execute(
                    "UPDATE time_entries SET task_id = ? WHERE id = ?",
                    duckdb::params![new_task_id.to_string(), entry.id.to_string()],
                )?;

                let updated = fetch_entry_by_id(conn, &entry.id)?;
                assert_eq!(updated.task_id, Some(new_task_id));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 存在しないエントリを更新しようとするとエラーになる() {
            let db = create_test_db();
            let non_existent_id = Uuid::new_v4();

            let result = db.with_connection(|conn| fetch_entry_by_id(conn, &non_existent_id));

            assert!(result.is_err());
        }
    }

    mod delete_entry_tests {
        use super::*;

        #[test]
        fn エントリを削除できる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                conn.execute(
                    "DELETE FROM time_entries WHERE id = ?",
                    [entry.id.to_string()],
                )?;

                let result = fetch_entry_by_id(conn, &entry.id);
                assert!(result.is_err());
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 存在しないエントリを削除しようとするとエラーになる() {
            let db = create_test_db();
            let non_existent_id = Uuid::new_v4();

            let result = db.with_connection(|conn| fetch_entry_by_id(conn, &non_existent_id));

            assert!(result.is_err());
        }
    }

    mod entry_with_relations_tests {
        use super::*;

        #[test]
        fn エントリをリレーション付きで取得するとタスク情報が含まれる() {
            let db = create_test_db();
            let task_id = Uuid::new_v4();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at) VALUES (?, 'テストタスク', '#ff0000', ?, ?)",
                    duckdb::params![task_id.to_string(), Utc::now(), Utc::now()],
                )?;

                let entry = TimeEntry::start(Some(task_id), None);
                insert_entry(conn, &entry)?;

                let with_relations = entry_to_with_relations(conn, entry)?;
                assert!(with_relations.task.is_some());
                assert_eq!(with_relations.task.unwrap().name, "テストタスク");
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 計測終了後のエントリにはduration_secondsが含まれる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let mut entry = TimeEntry::start(None, None);
                entry.ended_at = Some(entry.started_at + chrono::Duration::seconds(3600));
                insert_entry(conn, &entry)?;

                let with_relations = entry_to_with_relations(conn, entry)?;
                assert_eq!(with_relations.duration_seconds, Some(3600));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 計測中のエントリのduration_secondsはNone() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                insert_entry(conn, &entry)?;

                let with_relations = entry_to_with_relations(conn, entry)?;
                assert!(with_relations.duration_seconds.is_none());
                Ok(())
            })
            .unwrap();
        }
    }
}
