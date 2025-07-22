# ユニットテスト実装ガイド

## 目的

Librarian アプリケーションでのRustとフロントエンドユニットテストの実装方法とベストプラクティス

## Rustユニットテストの実装

### 基本構造

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test_input";

        // Act
        let result = function_to_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        // 非同期関数のテスト
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### セキュリティ関数のテスト例

```rust
#[test]
fn test_validate_path_security_valid_paths() {
    // 正常なパスのテスト
    assert!(validate_path_security("/Users/test/workspace").is_ok());
    assert!(validate_path_security("/home/user/documents").is_ok());
}

#[test]
fn test_validate_path_security_path_traversal() {
    // セキュリティ攻撃のテスト
    assert!(validate_path_security("/Users/test/../etc/passwd").is_err());
    assert!(validate_path_security("../etc/passwd").is_err());
}
```

### エラーハンドリングのテスト

```rust
#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    // .agent_libraryディレクトリを作成しない

    let result = AgentLibraryParser::parse(temp_dir.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
```

### テスト用ヘルパー関数

```rust
fn create_test_agent_library(dir: &Path) -> Result<()> {
    let agent_lib_dir = dir.join(".agent_library");
    fs::create_dir_all(&agent_lib_dir)?;

    let index_content = r#"
name: "Test Library"
description: "Test Agent Library"
version: "1.0.0"
mcp_endpoints:
  - id: "test_prompt"
    label: "Test Prompt"
    description: "A test prompt for unit testing"
    prompt_file: "test_prompt.md"
"#;
    fs::write(agent_lib_dir.join("agent_index.yml"), index_content)?;

    let prompt_content = "# Test Prompt\n\nThis is a test prompt.";
    fs::write(agent_lib_dir.join("test_prompt.md"), prompt_content)?;

    Ok(())
}
```

## フロントエンドユニットテストの実装

### 環境設定

#### vitest.config.ts

```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,ts}'],
  },
});
```

#### テストセットアップ

```typescript
// src/test/setup.ts
import '@testing-library/jest-dom';

const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

global.mockInvoke = mockInvoke;

beforeEach(() => {
  vi.clearAllMocks();
});
```

### Svelteコンポーネントのテスト

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Button from './Button.svelte';

describe('Button Component', () => {
  it('should render button with text', () => {
    render(Button, { props: { children: 'Click me' } });
    expect(screen.getByRole('button')).toHaveTextContent('Click me');
  });

  it('should be disabled when disabled prop is true', () => {
    render(Button, { props: { disabled: true, children: 'Disabled' } });
    expect(screen.getByRole('button')).toBeDisabled();
  });
});
```

### ストア（状態管理）のテスト

```typescript
describe('repositories store', () => {
  beforeEach(() => {
    repositories.set([]);
    vi.clearAllMocks();
  });

  it('should add repository successfully', async () => {
    mockInvoke.mockResolvedValue(undefined);

    await addRepository(mockRepository);

    const currentRepos = get(repositories);
    expect(currentRepos).toHaveLength(1);
    expect(mockInvoke).toHaveBeenCalledWith('add_repository_config', {
      repository: mockRepository,
    });
  });
});
```

### 非同期処理のテスト

```typescript
it('should handle async operations', async () => {
  mockInvoke.mockResolvedValue({ prompts: mockPrompts });

  render(PromptsView, { props: { repository: mockRepository } });

  await waitFor(() => {
    expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
  });
});
```

## テストのベストプラクティス

### 1. AAA パターンの使用

- **Arrange**: テストデータの準備
- **Act**: テスト対象の実行
- **Assert**: 結果の検証

### 2. テストの独立性

```rust
#[test]
fn test_independent() {
    let temp_dir = TempDir::new().unwrap(); // 各テストで新しい環境
    // テスト実行
    // TempDirは自動的にクリーンアップされる
}
```

### 3. 境界値テストの実装

```rust
#[test]
fn test_port_boundaries() {
    // 有効な境界値
    assert!(validate_port_security(9500).is_ok()); // 最小値
    assert!(validate_port_security(9599).is_ok()); // 最大値

    // 無効な境界値
    assert!(validate_port_security(9499).is_err()); // 最小値-1
    assert!(validate_port_security(9600).is_err()); // 最大値+1
}
```

### 4. エラーケースの網羅

```typescript
it('should handle error states', async () => {
  mockInvoke.mockRejectedValue(new Error('Network error'));

  render(Component);

  await waitFor(() => {
    expect(screen.getByText(/エラーが発生しました/)).toBeInTheDocument();
  });
});
```

## テスト実行コマンド

### Rustテスト

```bash
# 全テスト実行
cargo test

# 特定のテスト実行
cargo test test_validate_path_security

# 詳細出力
cargo test -- --nocapture
```

### フロントエンドテスト

```bash
# 全テスト実行
npm run test

# ウォッチモード
npm run test:watch

# カバレッジ測定
npm run test:coverage

# UI モード
npm run test:ui
```

## カバレッジ目標

### 最低要件

- **Rust**: 80%以上のカバレッジ
- **フロントエンド**: 70%以上のカバレッジ

### 重点テスト領域

1. **セキュリティ関連関数**: 100%カバレッジ必須
2. **エラーハンドリング**: 全パターンテスト
3. **ビジネスロジック**: 正常/異常ケース両方
4. **UI コンポーネント**: 状態変化の検証

## CI/CD統合

### GitHub Actions例

```yaml
- name: Run Rust tests
  run: cargo test

- name: Run Frontend tests
  run: |
    npm ci
    npm run test:run
```

## トラブルシューティング

### よくある問題と解決方法

#### 1. Tauriモックが動作しない

```typescript
// setup.tsでグローバルモックを正しく設定
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
```

#### 2. 非同期テストのタイムアウト

```typescript
// waitForのタイムアウトを調整
await waitFor(
  () => {
    expect(element).toBeInTheDocument();
  },
  { timeout: 5000 }
);
```

#### 3. テンポラリファイルのクリーンアップ

```rust
// TempDirを使用して自動クリーンアップ
let temp_dir = TempDir::new().unwrap();
// テスト実行
// ここでtemp_dirが自動的に削除される
```

## まとめ

- **包括的テスト**: 正常ケースと異常ケース両方をテスト
- **独立性**: 各テストは他のテストに依存しない
- **自動化**: CI/CDパイプラインに統合
- **保守性**: テストコードも本番コードと同じ品質で記述
