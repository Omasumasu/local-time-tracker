use chrono::{DateTime, Utc};
use duckdb::Connection;
use uuid::Uuid;

use crate::db::{
    Artifact, EntryArtifact, ExportData, ExportTimeEntry, ImportResult, Task,
};
use crate::error::AppResult;
use crate::AppState;

/// 全タスクを取得する
fn fetch_all_tasks(conn: &Connection) -> AppResult<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, folder_id, name, description, color, archived, created_at, updated_at FROM tasks ORDER BY created_at",
    )?;

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

/// 全成果物を取得する
fn fetch_all_artifacts(conn: &Connection) -> AppResult<Vec<Artifact>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, artifact_type, reference, metadata, created_at FROM artifacts ORDER BY created_at",
    )?;

    let rows = stmt.query_map([], |row| {
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

/// 全時間記録を取得する（エクスポート用）
fn fetch_all_entries(conn: &Connection) -> AppResult<Vec<ExportTimeEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, memo, created_at, updated_at FROM time_entries ORDER BY started_at",
    )?;

    let rows = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let task_id_str: Option<String> = row.get(1)?;
        let started_at: DateTime<Utc> = row.get(2)?;
        let ended_at: Option<DateTime<Utc>> = row.get(3)?;
        let created_at: DateTime<Utc> = row.get(5)?;
        let updated_at: DateTime<Utc> = row.get(6)?;

        let duration_seconds = ended_at.map(|ended| (ended - started_at).num_seconds());

        Ok(ExportTimeEntry {
            id: Uuid::parse_str(&id_str).unwrap(),
            task_id: task_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
            started_at,
            ended_at,
            duration_seconds,
            memo: row.get(4)?,
            created_at,
            updated_at,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

/// 全紐付けを取得する
fn fetch_all_entry_artifacts(conn: &Connection) -> AppResult<Vec<EntryArtifact>> {
    let mut stmt = conn.prepare("SELECT entry_id, artifact_id FROM entry_artifacts")?;

    let rows = stmt.query_map([], |row| {
        let entry_id_str: String = row.get(0)?;
        let artifact_id_str: String = row.get(1)?;

        Ok(EntryArtifact {
            entry_id: Uuid::parse_str(&entry_id_str).unwrap(),
            artifact_id: Uuid::parse_str(&artifact_id_str).unwrap(),
        })
    })?;

    let mut links = Vec::new();
    for row in rows {
        links.push(row?);
    }
    Ok(links)
}

/// データをエクスポートする
fn create_export_data(conn: &Connection) -> AppResult<ExportData> {
    Ok(ExportData {
        version: "1.0".to_string(),
        exported_at: Utc::now(),
        tasks: fetch_all_tasks(conn)?,
        artifacts: fetch_all_artifacts(conn)?,
        time_entries: fetch_all_entries(conn)?,
        entry_artifacts: fetch_all_entry_artifacts(conn)?,
    })
}

/// データをインポートする
fn import_export_data(conn: &Connection, data: &ExportData, merge: bool) -> AppResult<ImportResult> {
    if !merge {
        // マージしない場合は既存データを削除
        conn.execute("DELETE FROM entry_artifacts", [])?;
        conn.execute("DELETE FROM time_entries", [])?;
        conn.execute("DELETE FROM artifacts", [])?;
        conn.execute("DELETE FROM tasks", [])?;
    }

    let mut tasks_imported = 0;
    let mut entries_imported = 0;
    let mut artifacts_imported = 0;

    // タスクをインポート
    for task in &data.tasks {
        // マージモードの場合、既存のIDがあればスキップ
        if merge {
            let mut stmt =
                conn.prepare("SELECT COUNT(*) FROM tasks WHERE id = ?")?;
            let count: i64 = stmt.query_row([task.id.to_string()], |row| row.get(0))?;
            if count > 0 {
                continue;
            }
        }

        conn.execute(
            "INSERT INTO tasks (id, name, description, color, archived, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                task.id.to_string(),
                &task.name,
                &task.description,
                &task.color,
                task.archived,
                task.created_at,
                task.updated_at,
            ],
        )?;
        tasks_imported += 1;
    }

    // 成果物をインポート
    for artifact in &data.artifacts {
        if merge {
            let mut stmt =
                conn.prepare("SELECT COUNT(*) FROM artifacts WHERE id = ?")?;
            let count: i64 = stmt.query_row([artifact.id.to_string()], |row| row.get(0))?;
            if count > 0 {
                continue;
            }
        }

        conn.execute(
            "INSERT INTO artifacts (id, name, artifact_type, reference, metadata, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            duckdb::params![
                artifact.id.to_string(),
                &artifact.name,
                &artifact.artifact_type,
                &artifact.reference,
                artifact.metadata.as_ref().map(|m| m.to_string()),
                artifact.created_at,
            ],
        )?;
        artifacts_imported += 1;
    }

    // 時間記録をインポート
    for entry in &data.time_entries {
        if merge {
            let mut stmt =
                conn.prepare("SELECT COUNT(*) FROM time_entries WHERE id = ?")?;
            let count: i64 = stmt.query_row([entry.id.to_string()], |row| row.get(0))?;
            if count > 0 {
                continue;
            }
        }

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
        entries_imported += 1;
    }

    // 紐付けをインポート
    for link in &data.entry_artifacts {
        if merge {
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM entry_artifacts WHERE entry_id = ? AND artifact_id = ?",
            )?;
            let count: i64 = stmt
                .query_row([link.entry_id.to_string(), link.artifact_id.to_string()], |row| {
                    row.get(0)
                })?;
            if count > 0 {
                continue;
            }
        }

        // 紐付け先のエントリと成果物が存在するか確認
        let mut entry_stmt =
            conn.prepare("SELECT COUNT(*) FROM time_entries WHERE id = ?")?;
        let entry_exists: i64 =
            entry_stmt.query_row([link.entry_id.to_string()], |row| row.get(0))?;

        let mut artifact_stmt =
            conn.prepare("SELECT COUNT(*) FROM artifacts WHERE id = ?")?;
        let artifact_exists: i64 =
            artifact_stmt.query_row([link.artifact_id.to_string()], |row| row.get(0))?;

        if entry_exists > 0 && artifact_exists > 0 {
            conn.execute(
                "INSERT INTO entry_artifacts (entry_id, artifact_id) VALUES (?, ?)",
                duckdb::params![link.entry_id.to_string(), link.artifact_id.to_string()],
            )?;
        }
    }

    Ok(ImportResult {
        tasks_imported,
        entries_imported,
        artifacts_imported,
    })
}

/// JSONエクスポート
#[tauri::command]
pub fn export_data(state: tauri::State<AppState>) -> AppResult<ExportData> {
    state.db.with_connection(create_export_data)
}

/// JSONインポート
#[tauri::command]
pub fn import_data(
    state: tauri::State<AppState>,
    data: ExportData,
    merge: bool,
) -> AppResult<ImportResult> {
    state
        .db
        .with_connection(|conn| import_export_data(conn, &data, merge))
}

/// Parquetエクスポート
#[tauri::command]
pub fn export_parquet(state: tauri::State<AppState>, output_dir: String) -> AppResult<Vec<String>> {
    use std::path::Path;

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        std::fs::create_dir_all(output_path)?;
    }

    state.db.with_connection(|conn| {
        let mut exported_files = Vec::new();

        // tasks
        let tasks_path = output_path.join("tasks.parquet");
        conn.execute(
            &format!(
                "COPY tasks TO '{}' (FORMAT PARQUET)",
                tasks_path.to_string_lossy()
            ),
            [],
        )?;
        exported_files.push(tasks_path.to_string_lossy().to_string());

        // artifacts
        let artifacts_path = output_path.join("artifacts.parquet");
        conn.execute(
            &format!(
                "COPY artifacts TO '{}' (FORMAT PARQUET)",
                artifacts_path.to_string_lossy()
            ),
            [],
        )?;
        exported_files.push(artifacts_path.to_string_lossy().to_string());

        // time_entries
        let entries_path = output_path.join("time_entries.parquet");
        conn.execute(
            &format!(
                "COPY time_entries TO '{}' (FORMAT PARQUET)",
                entries_path.to_string_lossy()
            ),
            [],
        )?;
        exported_files.push(entries_path.to_string_lossy().to_string());

        // entry_artifacts
        let links_path = output_path.join("entry_artifacts.parquet");
        conn.execute(
            &format!(
                "COPY entry_artifacts TO '{}' (FORMAT PARQUET)",
                links_path.to_string_lossy()
            ),
            [],
        )?;
        exported_files.push(links_path.to_string_lossy().to_string());

        Ok(exported_files)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, TimeEntry};

    fn create_test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    mod export_data_tests {
        use super::*;

        #[test]
        fn 空のデータベースからエクスポートすると空のデータが返る() {
            let db = create_test_db();

            let export = db.with_connection(create_export_data).unwrap();

            assert_eq!(export.version, "1.0");
            assert!(export.tasks.is_empty());
            assert!(export.artifacts.is_empty());
            assert!(export.time_entries.is_empty());
            assert!(export.entry_artifacts.is_empty());
        }

        #[test]
        fn タスクがエクスポートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at)
                     VALUES (uuid(), 'テストタスク', '#000000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let export = db.with_connection(create_export_data).unwrap();

            assert_eq!(export.tasks.len(), 1);
            assert_eq!(export.tasks[0].name, "テストタスク");
        }

        #[test]
        fn 成果物がエクスポートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO artifacts (id, name, artifact_type, created_at)
                     VALUES (uuid(), 'テスト成果物', 'document', CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let export = db.with_connection(create_export_data).unwrap();

            assert_eq!(export.artifacts.len(), 1);
            assert_eq!(export.artifacts[0].name, "テスト成果物");
        }

        #[test]
        fn 時間記録がエクスポートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, Some("テストメモ".to_string()));
                conn.execute(
                    "INSERT INTO time_entries (id, started_at, memo, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?)",
                    duckdb::params![
                        entry.id.to_string(),
                        entry.started_at,
                        &entry.memo,
                        entry.created_at,
                        entry.updated_at,
                    ],
                )?;
                Ok(())
            })
            .unwrap();

            let export = db.with_connection(create_export_data).unwrap();

            assert_eq!(export.time_entries.len(), 1);
            assert_eq!(export.time_entries[0].memo, Some("テストメモ".to_string()));
        }

        #[test]
        fn 紐付けがエクスポートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let entry = TimeEntry::start(None, None);
                conn.execute(
                    "INSERT INTO time_entries (id, started_at, created_at, updated_at)
                     VALUES (?, ?, ?, ?)",
                    duckdb::params![
                        entry.id.to_string(),
                        entry.started_at,
                        entry.created_at,
                        entry.updated_at,
                    ],
                )?;

                let artifact_id = Uuid::new_v4();
                conn.execute(
                    "INSERT INTO artifacts (id, name, artifact_type, created_at)
                     VALUES (?, 'テスト', 'document', CURRENT_TIMESTAMP)",
                    [artifact_id.to_string()],
                )?;

                conn.execute(
                    "INSERT INTO entry_artifacts (entry_id, artifact_id) VALUES (?, ?)",
                    [entry.id.to_string(), artifact_id.to_string()],
                )?;

                Ok(())
            })
            .unwrap();

            let export = db.with_connection(create_export_data).unwrap();

            assert_eq!(export.entry_artifacts.len(), 1);
        }

        #[test]
        fn 終了した時間記録にはduration_secondsが含まれる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let mut entry = TimeEntry::start(None, None);
                entry.ended_at = Some(entry.started_at + chrono::Duration::seconds(3600));

                conn.execute(
                    "INSERT INTO time_entries (id, started_at, ended_at, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?)",
                    duckdb::params![
                        entry.id.to_string(),
                        entry.started_at,
                        entry.ended_at,
                        entry.created_at,
                        entry.updated_at,
                    ],
                )?;
                Ok(())
            })
            .unwrap();

            let export = db.with_connection(create_export_data).unwrap();

            assert_eq!(export.time_entries[0].duration_seconds, Some(3600));
        }
    }

    mod import_data_tests {
        use super::*;

        fn create_test_export_data() -> ExportData {
            let task = Task::new("インポートタスク".to_string(), None, None, None);
            let artifact = Artifact::new(
                "インポート成果物".to_string(),
                "document".to_string(),
                None,
                None,
            );
            let entry = TimeEntry::start(Some(task.id), Some("インポートメモ".to_string()));

            ExportData {
                version: "1.0".to_string(),
                exported_at: Utc::now(),
                tasks: vec![task.clone()],
                artifacts: vec![artifact.clone()],
                time_entries: vec![ExportTimeEntry {
                    id: entry.id,
                    task_id: entry.task_id,
                    started_at: entry.started_at,
                    ended_at: entry.ended_at,
                    duration_seconds: None,
                    memo: entry.memo,
                    created_at: entry.created_at,
                    updated_at: entry.updated_at,
                }],
                entry_artifacts: vec![EntryArtifact {
                    entry_id: entry.id,
                    artifact_id: artifact.id,
                }],
            }
        }

        #[test]
        fn データをインポートできる() {
            let db = create_test_db();
            let export_data = create_test_export_data();

            let result = db
                .with_connection(|conn| import_export_data(conn, &export_data, false))
                .unwrap();

            assert_eq!(result.tasks_imported, 1);
            assert_eq!(result.entries_imported, 1);
            assert_eq!(result.artifacts_imported, 1);
        }

        #[test]
        fn マージモードでは既存データが保持される() {
            let db = create_test_db();

            // 既存データを作成
            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at)
                     VALUES (uuid(), '既存タスク', '#000000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let export_data = create_test_export_data();

            db.with_connection(|conn| import_export_data(conn, &export_data, true))
                .unwrap();

            let tasks = db.with_connection(fetch_all_tasks).unwrap();
            assert_eq!(tasks.len(), 2); // 既存 + インポート
        }

        #[test]
        fn 非マージモードでは既存データが削除される() {
            let db = create_test_db();

            // 既存データを作成
            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at)
                     VALUES (uuid(), '既存タスク', '#000000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let export_data = create_test_export_data();

            db.with_connection(|conn| import_export_data(conn, &export_data, false))
                .unwrap();

            let tasks = db.with_connection(fetch_all_tasks).unwrap();
            assert_eq!(tasks.len(), 1); // インポートのみ
            assert_eq!(tasks[0].name, "インポートタスク");
        }

        #[test]
        fn マージモードで同じIDのデータはスキップされる() {
            let db = create_test_db();
            let export_data = create_test_export_data();

            // 1回目のインポート
            let result1 = db
                .with_connection(|conn| import_export_data(conn, &export_data, false))
                .unwrap();

            // 2回目のインポート（マージ）
            let result2 = db
                .with_connection(|conn| import_export_data(conn, &export_data, true))
                .unwrap();

            // 2回目は全てスキップされる
            assert_eq!(result1.tasks_imported, 1);
            assert_eq!(result2.tasks_imported, 0);
        }

        #[test]
        fn インポート後のデータが正しく取得できる() {
            let db = create_test_db();
            let export_data = create_test_export_data();

            db.with_connection(|conn| import_export_data(conn, &export_data, false))
                .unwrap();

            let tasks = db.with_connection(fetch_all_tasks).unwrap();
            let artifacts = db.with_connection(fetch_all_artifacts).unwrap();
            let entries = db.with_connection(fetch_all_entries).unwrap();

            assert_eq!(tasks[0].name, "インポートタスク");
            assert_eq!(artifacts[0].name, "インポート成果物");
            assert_eq!(entries[0].memo, Some("インポートメモ".to_string()));
        }
    }

    mod export_parquet_tests {
        use super::*;

        #[test]
        fn parquetファイルがエクスポートされる() {
            let db = create_test_db();
            let temp_dir = tempfile::tempdir().unwrap();
            let output_dir = temp_dir.path().to_string_lossy().to_string();

            let files = db
                .with_connection(|conn| {
                    // エクスポート実行
                    let mut exported_files = Vec::new();
                    let output_path = std::path::Path::new(&output_dir);

                    let tasks_path = output_path.join("tasks.parquet");
                    conn.execute(
                        &format!(
                            "COPY tasks TO '{}' (FORMAT PARQUET)",
                            tasks_path.to_string_lossy()
                        ),
                        [],
                    )?;
                    exported_files.push(tasks_path.to_string_lossy().to_string());

                    Ok(exported_files)
                })
                .unwrap();

            assert_eq!(files.len(), 1);
            assert!(std::path::Path::new(&files[0]).exists());
        }
    }
}
