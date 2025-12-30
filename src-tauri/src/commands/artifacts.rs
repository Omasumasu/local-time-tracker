use chrono::{DateTime, Utc};
use duckdb::Connection;
use uuid::Uuid;

use crate::db::{Artifact, CreateArtifact};
use crate::error::{AppError, AppResult};
use crate::AppState;

/// 成果物をDBに保存する
fn insert_artifact(conn: &Connection, artifact: &Artifact) -> AppResult<()> {
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
    Ok(())
}

/// 成果物一覧を取得する
fn fetch_artifacts(conn: &Connection, limit: Option<i64>) -> AppResult<Vec<Artifact>> {
    let sql = if let Some(lim) = limit {
        format!(
            "SELECT id, name, artifact_type, reference, metadata, created_at
             FROM artifacts ORDER BY created_at DESC LIMIT {}",
            lim
        )
    } else {
        "SELECT id, name, artifact_type, reference, metadata, created_at
         FROM artifacts ORDER BY created_at DESC"
            .to_string()
    };

    let mut stmt = conn.prepare(&sql)?;
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

/// IDで成果物を取得する
fn fetch_artifact_by_id(conn: &Connection, id: &Uuid) -> AppResult<Artifact> {
    let mut stmt = conn.prepare(
        "SELECT id, name, artifact_type, reference, metadata, created_at
         FROM artifacts WHERE id = ?",
    )?;

    let artifact = stmt
        .query_row([id.to_string()], |row| {
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
        })
        .map_err(|_| AppError::NotFound(format!("Artifact with id {} not found", id)))?;

    Ok(artifact)
}

/// エントリに成果物を紐付ける
fn link_artifact_to_entry(conn: &Connection, entry_id: &Uuid, artifact_id: &Uuid) -> AppResult<()> {
    conn.execute(
        "INSERT INTO entry_artifacts (entry_id, artifact_id) VALUES (?, ?)",
        duckdb::params![entry_id.to_string(), artifact_id.to_string()],
    )?;
    Ok(())
}

/// エントリから成果物の紐付けを解除する
fn unlink_artifact_from_entry(
    conn: &Connection,
    entry_id: &Uuid,
    artifact_id: &Uuid,
) -> AppResult<()> {
    let rows_affected = conn.execute(
        "DELETE FROM entry_artifacts WHERE entry_id = ? AND artifact_id = ?",
        duckdb::params![entry_id.to_string(), artifact_id.to_string()],
    )?;

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Link between entry and artifact not found".to_string(),
        ));
    }

    Ok(())
}

/// 成果物を作成する
#[tauri::command]
pub fn create_artifact(
    state: tauri::State<AppState>,
    artifact: CreateArtifact,
    entry_id: Option<String>,
) -> AppResult<Artifact> {
    if artifact.name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Artifact name cannot be empty".to_string(),
        ));
    }

    if artifact.artifact_type.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Artifact type cannot be empty".to_string(),
        ));
    }

    let new_artifact = Artifact::new(
        artifact.name,
        artifact.artifact_type,
        artifact.reference,
        artifact.metadata,
    );

    state.db.with_connection(|conn| {
        insert_artifact(conn, &new_artifact)?;

        // エントリIDが指定されていれば紐付ける
        if let Some(ref eid) = entry_id {
            let entry_uuid = Uuid::parse_str(eid)
                .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", eid)))?;
            link_artifact_to_entry(conn, &entry_uuid, &new_artifact.id)?;
        }

        Ok(new_artifact)
    })
}

/// 成果物一覧を取得する
#[tauri::command]
pub fn list_artifacts(state: tauri::State<AppState>, limit: Option<i64>) -> AppResult<Vec<Artifact>> {
    state.db.with_connection(|conn| fetch_artifacts(conn, limit))
}

/// エントリに成果物を紐付ける
#[tauri::command]
pub fn link_artifact(
    state: tauri::State<AppState>,
    entry_id: String,
    artifact_id: String,
) -> AppResult<()> {
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid entry UUID: {}", entry_id)))?;
    let artifact_uuid = Uuid::parse_str(&artifact_id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid artifact UUID: {}", artifact_id)))?;

    state.db.with_connection(|conn| {
        // 成果物が存在するか確認
        let _ = fetch_artifact_by_id(conn, &artifact_uuid)?;

        link_artifact_to_entry(conn, &entry_uuid, &artifact_uuid)
    })
}

/// エントリから成果物の紐付けを解除する
#[tauri::command]
pub fn unlink_artifact(
    state: tauri::State<AppState>,
    entry_id: String,
    artifact_id: String,
) -> AppResult<()> {
    let entry_uuid = Uuid::parse_str(&entry_id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid entry UUID: {}", entry_id)))?;
    let artifact_uuid = Uuid::parse_str(&artifact_id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid artifact UUID: {}", artifact_id)))?;

    state
        .db
        .with_connection(|conn| unlink_artifact_from_entry(conn, &entry_uuid, &artifact_uuid))
}

/// 成果物を削除する
#[tauri::command]
pub fn delete_artifact(state: tauri::State<AppState>, id: String) -> AppResult<()> {
    let artifact_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid UUID: {}", id)))?;

    state.db.with_connection(|conn| {
        // 成果物が存在するか確認
        let _ = fetch_artifact_by_id(conn, &artifact_id)?;

        // 紐付けを削除（ON DELETE CASCADEがあるが明示的に）
        conn.execute(
            "DELETE FROM entry_artifacts WHERE artifact_id = ?",
            [artifact_id.to_string()],
        )?;

        // 成果物を削除
        conn.execute("DELETE FROM artifacts WHERE id = ?", [artifact_id.to_string()])?;

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, TimeEntry};

    fn create_test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    mod create_artifact_tests {
        use super::*;

        #[test]
        fn 成果物を作成するとDBに保存される() {
            let db = create_test_db();
            let artifact = Artifact::new(
                "設計書.pdf".to_string(),
                "document".to_string(),
                Some("/docs/design.pdf".to_string()),
                None,
            );

            db.with_connection(|conn| {
                insert_artifact(conn, &artifact)?;
                let fetched = fetch_artifact_by_id(conn, &artifact.id)?;

                assert_eq!(fetched.name, "設計書.pdf");
                assert_eq!(fetched.artifact_type, "document");
                assert_eq!(fetched.reference, Some("/docs/design.pdf".to_string()));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn メタデータ付きの成果物を作成できる() {
            let db = create_test_db();
            let metadata = serde_json::json!({
                "size": 1024,
                "format": "pdf"
            });
            let artifact = Artifact::new(
                "ファイル".to_string(),
                "document".to_string(),
                None,
                Some(metadata.clone()),
            );

            db.with_connection(|conn| {
                insert_artifact(conn, &artifact)?;
                let fetched = fetch_artifact_by_id(conn, &artifact.id)?;

                assert_eq!(fetched.metadata, Some(metadata));
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 空の名前はバリデーションで検出される() {
            let create_artifact = CreateArtifact {
                name: "".to_string(),
                artifact_type: "document".to_string(),
                reference: None,
                metadata: None,
            };

            assert!(create_artifact.name.trim().is_empty());
        }

        #[test]
        fn 空の種別はバリデーションで検出される() {
            let create_artifact = CreateArtifact {
                name: "ファイル".to_string(),
                artifact_type: "".to_string(),
                reference: None,
                metadata: None,
            };

            assert!(create_artifact.artifact_type.trim().is_empty());
        }
    }

    mod list_artifacts_tests {
        use super::*;

        #[test]
        fn 空のデータベースから成果物一覧を取得すると空のベクターが返る() {
            let db = create_test_db();

            let artifacts = db
                .with_connection(|conn| fetch_artifacts(conn, None))
                .unwrap();

            assert!(artifacts.is_empty());
        }

        #[test]
        fn 作成した成果物が一覧に含まれる() {
            let db = create_test_db();
            let artifact =
                Artifact::new("テスト".to_string(), "code".to_string(), None, None);

            db.with_connection(|conn| {
                insert_artifact(conn, &artifact)?;
                let artifacts = fetch_artifacts(conn, None)?;

                assert_eq!(artifacts.len(), 1);
                assert_eq!(artifacts[0].name, "テスト");
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn limitを指定すると取得件数が制限される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                for i in 0..5 {
                    let artifact = Artifact::new(
                        format!("成果物{}", i),
                        "document".to_string(),
                        None,
                        None,
                    );
                    insert_artifact(conn, &artifact)?;
                }

                let artifacts = fetch_artifacts(conn, Some(3))?;
                assert_eq!(artifacts.len(), 3);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 成果物一覧は作成日時の降順でソートされる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                let artifact1 =
                    Artifact::new("成果物1".to_string(), "document".to_string(), None, None);
                insert_artifact(conn, &artifact1)?;

                std::thread::sleep(std::time::Duration::from_millis(10));

                let artifact2 =
                    Artifact::new("成果物2".to_string(), "document".to_string(), None, None);
                insert_artifact(conn, &artifact2)?;

                let artifacts = fetch_artifacts(conn, None)?;
                assert_eq!(artifacts.len(), 2);
                assert_eq!(artifacts[0].name, "成果物2"); // 新しい方が先
                assert_eq!(artifacts[1].name, "成果物1");
                Ok(())
            })
            .unwrap();
        }
    }

    mod link_artifact_tests {
        use super::*;

        #[test]
        fn エントリに成果物を紐付けられる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                // エントリを作成
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

                // 成果物を作成
                let artifact =
                    Artifact::new("テスト".to_string(), "document".to_string(), None, None);
                insert_artifact(conn, &artifact)?;

                // 紐付け
                link_artifact_to_entry(conn, &entry.id, &artifact.id)?;

                // 紐付けを確認
                let mut stmt = conn
                    .prepare("SELECT COUNT(*) FROM entry_artifacts WHERE entry_id = ? AND artifact_id = ?")?;
                let count: i64 =
                    stmt.query_row([entry.id.to_string(), artifact.id.to_string()], |row| {
                        row.get(0)
                    })?;

                assert_eq!(count, 1);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 同じ紐付けを複数回作成するとエラーになる() {
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

                let artifact =
                    Artifact::new("テスト".to_string(), "document".to_string(), None, None);
                insert_artifact(conn, &artifact)?;

                link_artifact_to_entry(conn, &entry.id, &artifact.id)?;
                let result = link_artifact_to_entry(conn, &entry.id, &artifact.id);

                // 重複挿入はエラーになる（PRIMARY KEY制約）
                assert!(result.is_err());
                Ok(())
            })
            .unwrap();
        }
    }

    mod unlink_artifact_tests {
        use super::*;

        #[test]
        fn エントリから成果物の紐付けを解除できる() {
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

                let artifact =
                    Artifact::new("テスト".to_string(), "document".to_string(), None, None);
                insert_artifact(conn, &artifact)?;

                link_artifact_to_entry(conn, &entry.id, &artifact.id)?;
                unlink_artifact_from_entry(conn, &entry.id, &artifact.id)?;

                // 紐付けが解除されたことを確認
                let mut stmt = conn
                    .prepare("SELECT COUNT(*) FROM entry_artifacts WHERE entry_id = ? AND artifact_id = ?")?;
                let count: i64 =
                    stmt.query_row([entry.id.to_string(), artifact.id.to_string()], |row| {
                        row.get(0)
                    })?;

                assert_eq!(count, 0);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 存在しない紐付けを解除しようとするとエラーになる() {
            let db = create_test_db();
            let entry_id = Uuid::new_v4();
            let artifact_id = Uuid::new_v4();

            let result = db.with_connection(|conn| {
                unlink_artifact_from_entry(conn, &entry_id, &artifact_id)
            });

            assert!(result.is_err());
        }
    }

    mod delete_artifact_tests {
        use super::*;

        #[test]
        fn 成果物を削除できる() {
            let db = create_test_db();
            let artifact =
                Artifact::new("テスト".to_string(), "document".to_string(), None, None);

            db.with_connection(|conn| {
                insert_artifact(conn, &artifact)?;
                conn.execute("DELETE FROM artifacts WHERE id = ?", [artifact.id.to_string()])?;

                let result = fetch_artifact_by_id(conn, &artifact.id);
                assert!(result.is_err());
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 成果物を削除すると紐付けも削除される() {
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

                let artifact =
                    Artifact::new("テスト".to_string(), "document".to_string(), None, None);
                insert_artifact(conn, &artifact)?;
                link_artifact_to_entry(conn, &entry.id, &artifact.id)?;

                // 紐付けを先に削除してから成果物を削除
                conn.execute(
                    "DELETE FROM entry_artifacts WHERE artifact_id = ?",
                    [artifact.id.to_string()],
                )?;
                conn.execute("DELETE FROM artifacts WHERE id = ?", [artifact.id.to_string()])?;

                // 紐付けも削除されていることを確認
                let mut stmt =
                    conn.prepare("SELECT COUNT(*) FROM entry_artifacts WHERE artifact_id = ?")?;
                let count: i64 = stmt.query_row([artifact.id.to_string()], |row| row.get(0))?;

                assert_eq!(count, 0);
                Ok(())
            })
            .unwrap();
        }

        #[test]
        fn 存在しない成果物を削除しようとするとエラーになる() {
            let db = create_test_db();
            let non_existent_id = Uuid::new_v4();

            let result = db.with_connection(|conn| fetch_artifact_by_id(conn, &non_existent_id));

            assert!(result.is_err());
        }
    }
}
