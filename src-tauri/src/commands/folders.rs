use chrono::{DateTime, Utc};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::AppState;

/// フォルダ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// フォルダ作成リクエスト
#[derive(Debug, Clone, Deserialize)]
pub struct CreateFolder {
    pub name: String,
    pub color: Option<String>,
}

/// フォルダ更新リクエスト
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateFolder {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

/// フォルダ一覧を取得する
fn fetch_folders(conn: &Connection) -> AppResult<Vec<Folder>> {
    let sql = r#"
        SELECT id, name, color, sort_order, created_at, updated_at
        FROM folders
        ORDER BY sort_order ASC, created_at ASC
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        Ok(Folder {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            name: row.get(1)?,
            color: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut folders = Vec::new();
    for row in rows {
        folders.push(row?);
    }
    Ok(folders)
}

/// フォルダを作成する
fn create_folder_impl(conn: &Connection, input: CreateFolder) -> AppResult<Folder> {
    if input.name.trim().is_empty() {
        return Err(AppError::InvalidInput("フォルダ名は必須です".to_string()));
    }

    let id = Uuid::new_v4();
    let now = Utc::now();
    let color = input.color.unwrap_or_else(|| "#6b7280".to_string());

    // Get max sort_order
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM folders", [], |row| row.get(0))
        .unwrap_or(0);

    let sql = r#"
        INSERT INTO folders (id, name, color, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;

    conn.execute(sql, [
        id.to_string(),
        input.name.trim().to_string(),
        color.clone(),
        (max_order + 1).to_string(),
        now.to_rfc3339(),
        now.to_rfc3339(),
    ])?;

    Ok(Folder {
        id,
        name: input.name.trim().to_string(),
        color,
        sort_order: max_order + 1,
        created_at: now,
        updated_at: now,
    })
}

/// フォルダを更新する
fn update_folder_impl(conn: &Connection, id: Uuid, input: UpdateFolder) -> AppResult<Folder> {
    let now = Utc::now();

    // Check folder exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE id = ?",
        [id.to_string()],
        |row| row.get(0),
    )?;

    if exists == 0 {
        return Err(AppError::NotFound("フォルダが見つかりません".to_string()));
    }

    let mut updates = vec!["updated_at = ?".to_string()];
    let mut params: Vec<String> = vec![now.to_rfc3339()];

    if let Some(name) = &input.name {
        if name.trim().is_empty() {
            return Err(AppError::InvalidInput("フォルダ名は必須です".to_string()));
        }
        updates.push("name = ?".to_string());
        params.push(name.trim().to_string());
    }

    if let Some(color) = &input.color {
        updates.push("color = ?".to_string());
        params.push(color.clone());
    }

    if let Some(sort_order) = input.sort_order {
        updates.push("sort_order = ?".to_string());
        params.push(sort_order.to_string());
    }

    params.push(id.to_string());

    let sql = format!(
        "UPDATE folders SET {} WHERE id = ?",
        updates.join(", ")
    );

    conn.execute(&sql, duckdb::params_from_iter(params))?;

    // Fetch updated folder
    let folder = conn.query_row(
        "SELECT id, name, color, sort_order, created_at, updated_at FROM folders WHERE id = ?",
        [id.to_string()],
        |row| {
            let id_str: String = row.get(0)?;
            Ok(Folder {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )?;

    Ok(folder)
}

/// フォルダを削除する（タスクのfolder_idはnullになる）
fn delete_folder_impl(conn: &Connection, id: Uuid) -> AppResult<()> {
    // Check folder exists
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE id = ?",
        [id.to_string()],
        |row| row.get(0),
    )?;

    if exists == 0 {
        return Err(AppError::NotFound("フォルダが見つかりません".to_string()));
    }

    // Set tasks' folder_id to null
    conn.execute(
        "UPDATE tasks SET folder_id = NULL WHERE folder_id = ?",
        [id.to_string()],
    )?;

    // Delete folder
    conn.execute("DELETE FROM folders WHERE id = ?", [id.to_string()])?;

    Ok(())
}

/// フォルダ一覧を取得する
#[tauri::command]
pub fn list_folders(state: tauri::State<AppState>) -> AppResult<Vec<Folder>> {
    state.db.with_connection(fetch_folders)
}

/// フォルダを作成する
#[tauri::command]
pub fn create_folder(state: tauri::State<AppState>, folder: CreateFolder) -> AppResult<Folder> {
    state.db.with_connection(|conn| create_folder_impl(conn, folder))
}

/// フォルダを更新する
#[tauri::command]
pub fn update_folder(
    state: tauri::State<AppState>,
    id: String,
    update: UpdateFolder,
) -> AppResult<Folder> {
    let uuid = Uuid::parse_str(&id).map_err(|_| AppError::InvalidInput("無効なIDです".to_string()))?;
    state.db.with_connection(|conn| update_folder_impl(conn, uuid, update))
}

/// フォルダを削除する
#[tauri::command]
pub fn delete_folder(state: tauri::State<AppState>, id: String) -> AppResult<()> {
    let uuid = Uuid::parse_str(&id).map_err(|_| AppError::InvalidInput("無効なIDです".to_string()))?;
    state.db.with_connection(|conn| delete_folder_impl(conn, uuid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn create_test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    #[test]
    fn フォルダを作成できる() {
        let db = create_test_db();

        let folder = db
            .with_connection(|conn| {
                create_folder_impl(conn, CreateFolder {
                    name: "開発".to_string(),
                    color: Some("#ff0000".to_string()),
                })
            })
            .unwrap();

        assert_eq!(folder.name, "開発");
        assert_eq!(folder.color, "#ff0000");
    }

    #[test]
    fn フォルダ一覧を取得できる() {
        let db = create_test_db();

        db.with_connection(|conn| {
            create_folder_impl(conn, CreateFolder {
                name: "フォルダ1".to_string(),
                color: None,
            })
        })
        .unwrap();

        db.with_connection(|conn| {
            create_folder_impl(conn, CreateFolder {
                name: "フォルダ2".to_string(),
                color: None,
            })
        })
        .unwrap();

        let folders = db.with_connection(fetch_folders).unwrap();
        assert_eq!(folders.len(), 2);
    }

    #[test]
    fn フォルダを更新できる() {
        let db = create_test_db();

        let folder = db
            .with_connection(|conn| {
                create_folder_impl(conn, CreateFolder {
                    name: "旧名".to_string(),
                    color: None,
                })
            })
            .unwrap();

        let updated = db
            .with_connection(|conn| {
                update_folder_impl(conn, folder.id, UpdateFolder {
                    name: Some("新名".to_string()),
                    color: None,
                    sort_order: None,
                })
            })
            .unwrap();

        assert_eq!(updated.name, "新名");
    }

    #[test]
    fn フォルダを削除できる() {
        let db = create_test_db();

        let folder = db
            .with_connection(|conn| {
                create_folder_impl(conn, CreateFolder {
                    name: "削除対象".to_string(),
                    color: None,
                })
            })
            .unwrap();

        db.with_connection(|conn| delete_folder_impl(conn, folder.id)).unwrap();

        let folders = db.with_connection(fetch_folders).unwrap();
        assert!(folders.is_empty());
    }
}
