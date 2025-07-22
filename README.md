# 📚 Librarian

![Librarian](./librarian_image.png)

> **プロンプトの管理とMCPサーバー統合のためのデスクトップアプリケーション**

Librarianは、AI開発者向けのプロンプト管理ツールです。MCP (Model Context Protocol)を提供し、Claude Codeをはじめとする様々なAIツールとシームレスに連携できます。

## ✨ 主な機能

### 🗂️ プロンプト管理

- **Agent Library形式**での構造化されたプロンプト管理
- **リアルタイム編集**とファイル同期
- **検索・フィルタリング**機能
- **カテゴリー別整理**

### 🔗 MCP統合

- **JSON-RPC 2.0**完全対応
- **マルチポート対応**（複数リポジトリ同時管理）
- **リアルタイムファイル監視**
- **Claude Code直接連携**

### 🖥️ デスクトップアプリ

- **クロスプラットフォーム対応**（~~Windows~~, macOS, ~~Linux~~）
- **ネイティブパフォーマンス**（Tauri v2）
- **モダンUI**（Svelte 5 + TailwindCSS）

## 🚀 クイックスタート

### 必要条件

- Node.js 18+
- Rust 1.70+
- Git

### インストール

```bash
# リポジトリをクローン
git clone <repository-url>
cd librarian

# 依存関係のインストール
npm install

# 開発サーバー起動
npm run tauri dev
```

### 初回セットアップ

1. **アプリケーション起動**

   ```bash
   npm run tauri dev
   ```

2. **リポジトリ追加**
   - 「新しいリポジトリを追加」をクリック
   - `.agent_library`フォルダを含むディレクトリを選択

3. **MCPサーバー起動**
   - リポジトリ選択後「MCP サーバー開始」をクリック
   - 自動的にポート（9500-9599）が割り当てられます

4. **Claude Code連携**
   ```bash
   claude mcp add -t http librarian-<repo-name> http://localhost:<port>
   ```

## 📁 Agent Library構造

Librarianは以下の構造の`.agent_library`ディレクトリを管理します：

```
.agent_library/
├── agent_index.yml          # プロンプト定義ファイル
├── PROMPT_NAME_1.md         # プロンプトファイル
├── PROMPT_NAME_2.md
└── ...
```

### agent_index.yml例

```yaml
name: 'My AI Prompts'
description: 'AI開発用プロンプトコレクション'
version: '1.0.0'

mcp_endpoints:
  - id: 'code_review'
    label: 'コードレビュー'
    description: 'コードの品質チェックとレビューを行う'
    prompt_file: 'PROMPT_NAME_1.md'
    trigger: 'review'
    category: 'development'
```

## 🔧 開発

### プロジェクト構成

```
librarian/
├── src/                     # Svelte フロントエンド
│   ├── lib/
│   │   ├── components/      # Svelteコンポーネント
│   │   ├── stores/          # 状態管理
│   │   └── types/           # TypeScript型定義
│   └── App.svelte
├── src-tauri/               # Rust バックエンド
│   ├── src/
│   │   ├── agent_library/   # Agent Library パーサー
│   │   ├── mcp/             # MCPサーバー実装
│   │   ├── persistence/     # 設定永続化
│   │   └── lib.rs
│   └── Cargo.toml
├── tests/                   # テスト
│   ├── e2e/                 # E2Eテスト（Playwright）
│   └── integration/         # 統合テスト
└── .agent_library/          # セルフホスティング用プロンプト
```

### 開発コマンド

```bash
# 開発サーバー起動
npm run dev

# ビルド
npm run build

# テスト実行
npm run test              # ユニットテスト
npm run test:e2e          # E2Eテスト
cargo test                # Rustテスト

# コード品質チェック
npm run quality           # ESLint + Prettier + svelte-check
cargo clippy              # Rust linting
```

### テスト

#### フロントエンド

- **ユニットテスト**: Vitest + @testing-library/svelte
- **E2Eテスト**: Playwright

#### バックエンド

- **ユニットテスト**: Rust標準テストフレームワーク
- **統合テスト**: Axum + Tower + Hyper

```bash
# 全テスト実行
npm run test:run && cargo test --all
```

## 📐 アーキテクチャ

### 技術スタック

- **フロントエンド**: Svelte 5 + TypeScript + TailwindCSS
- **バックエンド**: Rust + Tauri v2 + Axum
- **プロトコル**: JSON-RPC 2.0 over HTTP
- **ファイル監視**: notify crate
- **テスト**: Vitest + Playwright + Rust標準

### セキュリティ

- **パストラバーサル防止**
- **ポート範囲制限**（9500-9599）
- **CORS制限**（localhost限定）
- **ファイルサイズ制限**（1MB）
- **絶対パス強制**

## 🔌 MCP連携

### サポートされるMCPメソッド

- `initialize` - プロトコル初期化
- `initialized` - 初期化完了通知
- `prompts/list` - プロンプト一覧取得
- `prompts/get` - 特定プロンプト取得
- `resources/list` - リソース一覧取得
- `resources/read` - リソース内容取得

### Claude Code連携例

```bash
# MCPサーバー追加
claude mcp add -t http my-prompts http://localhost:9500

# プロンプト使用（claude cliで利用可能）
claude chat --prompt code_review "レビューしてください"
```

## 🤝 コントリビューション

1. このリポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

### 開発ガイドライン

- **コミットメッセージ**: [Conventional Commits](https://conventionalcommits.org/)に従う
- **コードスタイル**: Prettier + ESLint + rustfmt
- **テスト**: 新機能には必ずテストを追加
- **ドキュメント**: 重要な変更には`.agent_library`ガイドを更新

## 📄 ライセンス

このプロジェクトは[MIT License](LICENSE)の下でライセンスされています。

## 🙏 謝辞

- [Tauri](https://tauri.app/) - クロスプラットフォームデスクトップアプリフレームワーク
- [Svelte](https://svelte.dev/) - リアクティブUIフレームワーク
- [Model Context Protocol](https://modelcontextprotocol.io/) - AI統合プロトコル
- [Claude Code](https://claude.ai/code) - AI開発ツール

---

**📚 Librarian で、AIプロンプトの管理を簡単に。**
