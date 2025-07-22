# フロントエンド品質管理ガイド

## 概要

Svelte 5 + TailwindCSS + TypeScriptプロジェクトでの品質管理ツールの設定と使用方法

## 必要なツール

### 1. svelte-check

TypeScriptとSvelteの型チェックツール

```bash
# インストール
npm install -D svelte-check

# 実行
npm run svelte-check
```

### 2. ESLint

JavaScriptとTypeScriptのコード品質チェック

```bash
# Svelte用ESLint設定のインストール
npm install -D eslint @typescript-eslint/eslint-plugin @typescript-eslint/parser eslint-plugin-svelte

# 実行
npm run lint
```

### 3. Prettier

コードフォーマッター

```bash
# Svelte用Prettier設定のインストール
npm install -D prettier prettier-plugin-svelte

# 実行
npm run format
```

## package.json スクリプト設定

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "svelte-check": "svelte-check --tsconfig ./tsconfig.json",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "quality": "npm run svelte-check && npm run lint && npm run format:check"
  }
}
```

## ESLint設定 (.eslintrc.cjs)

```javascript
module.exports = {
  root: true,
  extends: ['eslint:recommended', '@typescript-eslint/recommended', 'plugin:svelte/recommended'],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  parserOptions: {
    sourceType: 'module',
    ecmaVersion: 2020,
    extraFileExtensions: ['.svelte'],
  },
  env: {
    browser: true,
    es2017: true,
    node: true,
  },
  overrides: [
    {
      files: ['*.svelte'],
      parser: 'svelte-eslint-parser',
      parserOptions: {
        parser: '@typescript-eslint/parser',
      },
    },
  ],
};
```

## Prettier設定 (.prettierrc)

```json
{
  "useTabs": false,
  "singleQuote": true,
  "trailingComma": "es5",
  "printWidth": 100,
  "plugins": ["prettier-plugin-svelte"],
  "overrides": [
    {
      "files": "*.svelte",
      "options": {
        "parser": "svelte"
      }
    }
  ]
}
```

## 開発ワークフロー

### コミット前チェック

```bash
# 品質チェック実行
npm run quality

# 問題があれば修正
npm run lint:fix
npm run format
```

### CI/CD用チェック

```bash
# ビルドチェック
npm run build

# 型チェック
npm run svelte-check

# リンターチェック
npm run lint

# フォーマットチェック
npm run format:check
```

## よくある問題と解決方法

### 1. Svelte 5 での型エラー

```typescript
// ❌ 古いSvelte API
import Component from './Component.svelte';
new Component({ target: document.body });

// ✅ Svelte 5 API
import { mount } from 'svelte';
import Component from './Component.svelte';
mount(Component, { target: document.body });
```

### 2. TailwindCSS クラスの警告

```svelte
<!-- ESLintでTailwindクラスが未使用として警告される場合 -->
<!-- .eslintrc.cjs に以下を追加 -->
{
  "rules": {
    "svelte/no-unused-class-name": "off"
  }
}
```

### 3. TypeScript strict モード

```json
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

## ベストプラクティス

1. **定期的な品質チェック**: `npm run quality`を定期実行
2. **自動フォーマット**: エディタでPrettierの自動実行を設定
3. **型安全性**: TypeScriptの型注釈を積極的に使用
4. **コンポーネント設計**: 単一責任の原則でコンポーネントを分割
5. **props型定義**: Svelteコンポーネントのpropsは必ず型定義

## 品質メトリクス

- ESLintエラー: 0個
- TypeScript型エラー: 0個
- Prettierフォーマット違反: 0個
- ビルドエラー: 0個
- 未使用import: 0個
