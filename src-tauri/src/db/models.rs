use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// タスク（作業内容）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// タスク作成用DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub folder_id: Option<Uuid>,
}

/// タスク更新用DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTask {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
}

/// 成果物
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    pub id: Uuid,
    pub name: String,
    pub artifact_type: String,
    pub reference: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 成果物作成用DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArtifact {
    pub name: String,
    pub artifact_type: String,
    pub reference: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// 時間記録
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeEntry {
    pub id: Uuid,
    pub task_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 時間記録（リレーション付き）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryWithRelations {
    pub id: Uuid,
    pub task_id: Option<Uuid>,
    pub task: Option<Task>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub memo: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 時間記録更新用DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateEntry {
    pub task_id: Option<Option<Uuid>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<Option<DateTime<Utc>>>,
    pub memo: Option<String>,
}

/// エントリ検索条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntryFilter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub task_id: Option<Uuid>,
    pub limit: Option<i64>,
}

/// エクスポートデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub tasks: Vec<Task>,
    pub artifacts: Vec<Artifact>,
    pub time_entries: Vec<ExportTimeEntry>,
    pub entry_artifacts: Vec<EntryArtifact>,
}

/// エクスポート用の時間記録（duration_seconds付き）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTimeEntry {
    pub id: Uuid,
    pub task_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// エントリと成果物の紐付け
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntryArtifact {
    pub entry_id: Uuid,
    pub artifact_id: Uuid,
}

/// インポート結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub tasks_imported: usize,
    pub entries_imported: usize,
    pub artifacts_imported: usize,
}

impl Task {
    /// 新しいタスクを作成する
    pub fn new(name: String, description: Option<String>, color: Option<String>, folder_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            folder_id,
            name,
            description,
            color: color.unwrap_or_else(|| "#3b82f6".to_string()),
            archived: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// カラーコードが有効な形式かチェックする
    pub fn is_valid_color(color: &str) -> bool {
        if color.len() != 7 {
            return false;
        }
        if !color.starts_with('#') {
            return false;
        }
        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl TimeEntry {
    /// 新しい計測を開始する
    pub fn start(task_id: Option<Uuid>, memo: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id,
            started_at: now,
            ended_at: None,
            memo,
            created_at: now,
            updated_at: now,
        }
    }

    /// 計測中かどうかを判定する
    pub fn is_running(&self) -> bool {
        self.ended_at.is_none()
    }

    /// 経過秒数を計算する
    pub fn duration_seconds(&self) -> Option<i64> {
        self.ended_at.map(|ended| {
            (ended - self.started_at).num_seconds()
        })
    }
}

impl Artifact {
    /// 新しい成果物を作成する
    pub fn new(
        name: String,
        artifact_type: String,
        reference: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            artifact_type,
            reference,
            metadata,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod task_tests {
        use super::*;

        #[test]
        fn タスクを作成するとUUIDが生成される() {
            let task = Task::new("テスト作業".to_string(), None, None, None);

            assert!(!task.id.is_nil());
            assert_eq!(task.name, "テスト作業");
        }

        #[test]
        fn タスクを作成すると説明がオプションで設定できる() {
            let task = Task::new(
                "設計作業".to_string(),
                Some("システム設計を行う".to_string()),
                None,
                None,
            );

            assert_eq!(task.description, Some("システム設計を行う".to_string()));
        }

        #[test]
        fn タスクを作成するとデフォルトの青色が設定される() {
            let task = Task::new("テスト".to_string(), None, None, None);

            assert_eq!(task.color, "#3b82f6");
        }

        #[test]
        fn タスクを作成するとカスタムカラーを設定できる() {
            let task = Task::new("テスト".to_string(), None, Some("#ff0000".to_string()), None);

            assert_eq!(task.color, "#ff0000");
        }

        #[test]
        fn タスクを作成すると初期状態ではアーカイブされていない() {
            let task = Task::new("テスト".to_string(), None, None, None);

            assert!(!task.archived);
        }

        #[test]
        fn タスクを作成すると作成日時と更新日時が同じになる() {
            let task = Task::new("テスト".to_string(), None, None, None);

            assert_eq!(task.created_at, task.updated_at);
        }

        #[test]
        fn タスクにフォルダIDを設定できる() {
            let folder_id = Uuid::new_v4();
            let task = Task::new("テスト".to_string(), None, None, Some(folder_id));

            assert_eq!(task.folder_id, Some(folder_id));
        }

        #[test]
        fn 有効なカラーコードを検証できる() {
            assert!(Task::is_valid_color("#000000"));
            assert!(Task::is_valid_color("#ffffff"));
            assert!(Task::is_valid_color("#3b82f6"));
            assert!(Task::is_valid_color("#AABBCC"));
        }

        #[test]
        fn 無効なカラーコードを検出できる() {
            assert!(!Task::is_valid_color(""));
            assert!(!Task::is_valid_color("000000"));
            assert!(!Task::is_valid_color("#00000"));
            assert!(!Task::is_valid_color("#0000000"));
            assert!(!Task::is_valid_color("#gggggg"));
            assert!(!Task::is_valid_color("red"));
        }

        #[test]
        fn タスクをJSONにシリアライズできる() {
            let task = Task::new("テスト".to_string(), None, None, None);
            let json = serde_json::to_string(&task);

            assert!(json.is_ok());
        }

        #[test]
        fn JSONからタスクをデシリアライズできる() {
            let task = Task::new("テスト".to_string(), Some("説明".to_string()), None, None);
            let json = serde_json::to_string(&task).unwrap();
            let deserialized: Task = serde_json::from_str(&json).unwrap();

            assert_eq!(task, deserialized);
        }
    }

    mod time_entry_tests {
        use super::*;

        #[test]
        fn 計測を開始するとUUIDが生成される() {
            let entry = TimeEntry::start(None, None);

            assert!(!entry.id.is_nil());
        }

        #[test]
        fn 計測を開始するとタスクIDをオプションで紐付けできる() {
            let task_id = Uuid::new_v4();
            let entry = TimeEntry::start(Some(task_id), None);

            assert_eq!(entry.task_id, Some(task_id));
        }

        #[test]
        fn 計測を開始するとメモをオプションで設定できる() {
            let entry = TimeEntry::start(None, Some("作業メモ".to_string()));

            assert_eq!(entry.memo, Some("作業メモ".to_string()));
        }

        #[test]
        fn 計測開始時は終了時刻がNoneになる() {
            let entry = TimeEntry::start(None, None);

            assert!(entry.ended_at.is_none());
        }

        #[test]
        fn 計測中の場合is_runningがtrueを返す() {
            let entry = TimeEntry::start(None, None);

            assert!(entry.is_running());
        }

        #[test]
        fn 計測停止後はis_runningがfalseを返す() {
            let mut entry = TimeEntry::start(None, None);
            entry.ended_at = Some(Utc::now());

            assert!(!entry.is_running());
        }

        #[test]
        fn 計測中のduration_secondsはNoneを返す() {
            let entry = TimeEntry::start(None, None);

            assert!(entry.duration_seconds().is_none());
        }

        #[test]
        fn 計測停止後のduration_secondsは経過秒数を返す() {
            let mut entry = TimeEntry::start(None, None);
            let start = entry.started_at;
            entry.ended_at = Some(start + chrono::Duration::seconds(3600));

            assert_eq!(entry.duration_seconds(), Some(3600));
        }

        #[test]
        fn 時間記録をJSONにシリアライズできる() {
            let entry = TimeEntry::start(None, Some("テスト".to_string()));
            let json = serde_json::to_string(&entry);

            assert!(json.is_ok());
        }

        #[test]
        fn JSONから時間記録をデシリアライズできる() {
            let entry = TimeEntry::start(Some(Uuid::new_v4()), Some("メモ".to_string()));
            let json = serde_json::to_string(&entry).unwrap();
            let deserialized: TimeEntry = serde_json::from_str(&json).unwrap();

            assert_eq!(entry, deserialized);
        }
    }

    mod artifact_tests {
        use super::*;

        #[test]
        fn 成果物を作成するとUUIDが生成される() {
            let artifact = Artifact::new(
                "設計書.pdf".to_string(),
                "document".to_string(),
                None,
                None,
            );

            assert!(!artifact.id.is_nil());
        }

        #[test]
        fn 成果物に名前と種別を設定できる() {
            let artifact = Artifact::new(
                "設計書.pdf".to_string(),
                "document".to_string(),
                None,
                None,
            );

            assert_eq!(artifact.name, "設計書.pdf");
            assert_eq!(artifact.artifact_type, "document");
        }

        #[test]
        fn 成果物に参照先を設定できる() {
            let artifact = Artifact::new(
                "コード".to_string(),
                "code".to_string(),
                Some("https://github.com/example/repo".to_string()),
                None,
            );

            assert_eq!(
                artifact.reference,
                Some("https://github.com/example/repo".to_string())
            );
        }

        #[test]
        fn 成果物にメタデータをJSON形式で設定できる() {
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

            assert_eq!(artifact.metadata, Some(metadata));
        }

        #[test]
        fn 成果物をJSONにシリアライズできる() {
            let artifact = Artifact::new(
                "テスト".to_string(),
                "code".to_string(),
                Some("/path/to/file".to_string()),
                None,
            );
            let json = serde_json::to_string(&artifact);

            assert!(json.is_ok());
        }

        #[test]
        fn JSONから成果物をデシリアライズできる() {
            let artifact = Artifact::new(
                "テスト".to_string(),
                "document".to_string(),
                Some("/path".to_string()),
                Some(serde_json::json!({"key": "value"})),
            );
            let json = serde_json::to_string(&artifact).unwrap();
            let deserialized: Artifact = serde_json::from_str(&json).unwrap();

            assert_eq!(artifact, deserialized);
        }
    }

    mod export_data_tests {
        use super::*;

        #[test]
        fn エクスポートデータをJSONにシリアライズできる() {
            let export_data = ExportData {
                version: "1.0".to_string(),
                exported_at: Utc::now(),
                tasks: vec![],
                artifacts: vec![],
                time_entries: vec![],
                entry_artifacts: vec![],
            };
            let json = serde_json::to_string(&export_data);

            assert!(json.is_ok());
        }

        #[test]
        fn JSONからエクスポートデータをデシリアライズできる() {
            let json = r#"{
                "version": "1.0",
                "exported_at": "2024-12-30T10:00:00Z",
                "tasks": [],
                "artifacts": [],
                "time_entries": [],
                "entry_artifacts": []
            }"#;
            let result: Result<ExportData, _> = serde_json::from_str(json);

            assert!(result.is_ok());
            assert_eq!(result.unwrap().version, "1.0");
        }
    }

    mod entry_artifact_tests {
        use super::*;

        #[test]
        fn エントリと成果物の紐付けを作成できる() {
            let entry_id = Uuid::new_v4();
            let artifact_id = Uuid::new_v4();
            let link = EntryArtifact {
                entry_id,
                artifact_id,
            };

            assert_eq!(link.entry_id, entry_id);
            assert_eq!(link.artifact_id, artifact_id);
        }

        #[test]
        fn 紐付けをJSONにシリアライズできる() {
            let link = EntryArtifact {
                entry_id: Uuid::new_v4(),
                artifact_id: Uuid::new_v4(),
            };
            let json = serde_json::to_string(&link);

            assert!(json.is_ok());
        }
    }
}
