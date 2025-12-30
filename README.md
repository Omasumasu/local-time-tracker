# Local Time Tracker

ローカルファーストの時間管理アプリケーション。Tauri v2、React、DuckDBで構築。

## 機能

- **時間計測**: タイマーで作業時間を記録
- **タスク管理**: タスクを作成し、カスタムカラーで管理
- **フォルダシステム**: タスクをフォルダで整理・分類
- **月次レポート**: チャート付きの詳細レポート（タスク別内訳、日別推移）
- **データエクスポート**: JSON形式またはPNG画像でエクスポート
- **オフラインファースト**: DuckDBでローカルにデータを保存

## 技術スタック

- **フロントエンド**: React 18, TypeScript, Vite
- **UI**: shadcn/ui, Tailwind CSS v4
- **アニメーション**: Framer Motion
- **チャート**: Chart.js
- **バックエンド**: Tauri v2 (Rust)
- **データベース**: DuckDB（組み込み型）

## セットアップ

### 必要な環境

- **Node.js**: 18以上
- **Rust**: 最新の安定版
- **npm**: パッケージマネージャー

### Rustのインストール

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# インストール後、シェルを再起動するか以下を実行
source $HOME/.cargo/env

# バージョン確認
rustc --version
```

### Node.jsのインストール

```bash
# Homebrewを使う場合（macOS）
brew install node

# または asdf を使う場合
asdf plugin add nodejs
asdf install nodejs 20.10.0
asdf global nodejs 20.10.0

# バージョン確認
node --version
npm --version
```

### プロジェクトのセットアップ

```bash
# リポジトリをクローン
git clone https://github.com/your-username/local-time-tracker.git
cd local-time-tracker

# 依存パッケージをインストール
npm install

# 開発モードで起動
npm run tauri dev
```

### ビルド

```bash
# 本番用ビルド
npm run tauri build

# ビルド成果物は以下に出力されます
# macOS: src-tauri/target/release/bundle/dmg/
# Windows: src-tauri/target/release/bundle/msi/
# Linux: src-tauri/target/release/bundle/deb/
```

## プロジェクト構成

```
local-time-tracker/
├── src/                    # Reactフロントエンド
│   ├── components/         # UIコンポーネント
│   │   ├── ui/             # shadcn/ui コンポーネント
│   │   ├── Timer.tsx       # タイマー
│   │   ├── Timeline.tsx    # タイムライン
│   │   ├── TaskManager.tsx # タスク・フォルダ管理
│   │   └── Report.tsx      # 月次レポート
│   ├── hooks/              # カスタムフック
│   │   └── useStore.ts     # 状態管理
│   ├── api/                # Tauri APIバインディング
│   └── types/              # TypeScript型定義
├── src-tauri/              # Tauriバックエンド（Rust）
│   ├── src/
│   │   ├── commands/       # Tauriコマンド
│   │   │   ├── tasks.rs    # タスクCRUD
│   │   │   ├── entries.rs  # 時間記録
│   │   │   ├── folders.rs  # フォルダCRUD
│   │   │   └── reports.rs  # レポート生成
│   │   ├── db/             # データベース層
│   │   │   ├── connection.rs
│   │   │   ├── migrations.rs
│   │   │   └── models.rs
│   │   └── error.rs        # エラーハンドリング
│   └── migrations/         # SQLマイグレーション
├── package.json
└── README.md
```

## 開発コマンド

```bash
# 開発サーバー起動
npm run tauri dev

# フロントエンドのみ起動
npm run dev

# Rustテスト実行
cd src-tauri && cargo test

# TypeScript型チェック
npm run type-check

# 本番ビルド
npm run tauri build
```

## データの保存場所

アプリケーションデータは以下の場所に保存されます：

- **macOS**: `~/Library/Application Support/com.example.local-time-tracker/`
- **Windows**: `%APPDATA%\com.example.local-time-tracker\`
- **Linux**: `~/.local/share/com.example.local-time-tracker/`

データベースファイル: `time_tracker.db`

## ライセンス

MIT
