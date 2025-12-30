# CLAUDE.md - Local Time Tracker

## プロジェクト概要

ローカルファーストの作業時間トラッキングアプリケーション。Tauri v2 + DuckDB + TypeScript (Vanilla) で構築。

## 技術スタック

- **フロントエンド**: TypeScript (Vanilla), Vite 6.0
- **バックエンド**: Rust (Tauri v2)
- **データベース**: DuckDB (ローカルファイルDB)
- **テスト**: Vitest (フロントエンド), cargo test (バックエンド)

## ディレクトリ構成

```
local-time-tracker/
├── src/                    # TypeScriptフロントエンド
│   ├── api/               # Tauriコマンド呼び出しラッパー
│   ├── store/             # 状態管理
│   ├── components/        # UIコンポーネント
│   └── types/             # 型定義
├── src-tauri/             # Rustバックエンド
│   ├── src/
│   │   ├── commands/      # Tauriコマンド (tasks, entries, artifacts, export)
│   │   ├── db/            # DB接続、モデル、マイグレーション
│   │   └── error.rs       # エラー型定義
│   └── migrations/        # SQLマイグレーションファイル
└── docs/                  # 設計ドキュメント
```

## 開発ルール

### TDD (テスト駆動開発)

- **テストファースト**: 実装前に必ずテストを書く
- **テスト名は仕様**: 日本語でテスト名を記述し、仕様を表現する
  - 例: `空のタスク名はエラーになる()`, `タスク一覧は作成日時の降順でソートされる()`

### DB設計

- **タイムスタンプ**: `TIMESTAMPTZ`型を使用（VARCHARは使わない）
- **UUID**: DuckDB互換性のため`VARCHAR`で保存
- **chrono連携**: `duckdb`クレートの`chrono`フィーチャーで`DateTime<Utc>`を直接バインド/フェッチ

```rust
// Good: DateTime<Utc>を直接使用
conn.execute(
    "INSERT INTO tasks (..., created_at) VALUES (..., ?)",
    duckdb::params![task.created_at],  // DateTime<Utc>
)?;

// Bad: 文字列変換は避ける
// task.created_at.to_rfc3339()
```

## よく使うコマンド

```bash
# Rustバックエンドのテスト
cd src-tauri && cargo test

# 特定のテストモジュールのみ実行
cargo test --lib commands::tasks

# ビルド確認
cargo build

# フロントエンド開発サーバー
npm run dev

# フロントエンドテスト
npm test

# Tauri開発モード
npm run tauri dev
```

## データベーススキーマ

### tasks (作業内容)
| カラム | 型 | 説明 |
|--------|------|------|
| id | VARCHAR | UUID (PK) |
| name | VARCHAR | タスク名 |
| description | TEXT | 説明 |
| color | VARCHAR(7) | カラーコード (#RRGGBB) |
| archived | BOOLEAN | アーカイブフラグ |
| created_at | TIMESTAMPTZ | 作成日時 |
| updated_at | TIMESTAMPTZ | 更新日時 |

### time_entries (作業記録)
| カラム | 型 | 説明 |
|--------|------|------|
| id | VARCHAR | UUID (PK) |
| task_id | VARCHAR | タスクID (FK, nullable) |
| started_at | TIMESTAMPTZ | 開始日時 |
| ended_at | TIMESTAMPTZ | 終了日時 (nullable) |
| memo | TEXT | メモ |
| created_at | TIMESTAMPTZ | 作成日時 |
| updated_at | TIMESTAMPTZ | 更新日時 |

### artifacts (成果物)
| カラム | 型 | 説明 |
|--------|------|------|
| id | VARCHAR | UUID (PK) |
| name | VARCHAR | 成果物名 |
| artifact_type | VARCHAR(50) | 種別 (url, file, commit, etc.) |
| reference | TEXT | 参照先 |
| metadata | JSON | 追加情報 |
| created_at | TIMESTAMPTZ | 作成日時 |

### entry_artifacts (紐付け)
| カラム | 型 | 説明 |
|--------|------|------|
| entry_id | VARCHAR | エントリID (PK) |
| artifact_id | VARCHAR | 成果物ID (PK) |

## API仕様

詳細は `docs/time-tracker-design.md` を参照。

### Tasks
- `list_tasks(include_archived: bool)` - タスク一覧取得
- `create_task(task: CreateTask)` - タスク作成
- `update_task(id: String, update: UpdateTask)` - タスク更新
- `archive_task(id: String, archived: bool)` - アーカイブ/復元

### Entries
- `list_entries(from?, to?, task_id?)` - エントリ一覧取得
- `get_running_entry()` - 実行中エントリ取得
- `start_entry(task_id?, memo?)` - 計測開始
- `stop_entry(id: String, memo?)` - 計測停止
- `update_entry(id: String, update: UpdateEntry)` - エントリ更新
- `delete_entry(id: String)` - エントリ削除

### Artifacts
- `create_artifact(artifact: CreateArtifact)` - 成果物作成
- `list_artifacts_for_entry(entry_id: String)` - エントリの成果物一覧
- `link_artifact(entry_id: String, artifact_id: String)` - 紐付け
- `unlink_artifact(entry_id: String, artifact_id: String)` - 紐付け解除

### Export/Import
- `export_data(format: "json" | "csv")` - データエクスポート
- `import_data(data: String, format: "json")` - データインポート
