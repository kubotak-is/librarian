# Svelte 5 コンポーネント開発ガイド

## 概要

Svelte 5でのコンポーネント作成、Runes、state管理のベストプラクティス

## Svelte 5 の新機能

### 1. Runes（ルーン）システム

Svelte 5の新しいリアクティビティシステム

```javascript
// $state - リアクティブな状態
let count = $state(0);

// $derived - 派生状態
let doubled = $derived(count * 2);

// $effect - 副作用
$effect(() => {
  console.log('Count changed:', count);
});

// $props - プロパティ受け取り
let { title, items = [] } = $props();
```

### 2. 新しいコンポーネントAPI

```javascript
// 旧来のAPIは非推奨
// import Component from './Component.svelte';
// new Component({ target: document.body });

// 新しいmount API
import { mount } from 'svelte';
import Component from './Component.svelte';

mount(Component, {
  target: document.body,
  props: { title: 'Hello' },
});
```

## コンポーネントパターン

### 1. 基本コンポーネント

```svelte
<!-- Button.svelte -->
<script lang="ts">
  interface Props {
    variant?: 'primary' | 'secondary' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    onclick?: () => void;
    children: Snippet;
  }

  let { variant = 'primary', size = 'md', disabled = false, onclick, children }: Props = $props();

  const baseClasses = 'px-4 py-2 rounded font-medium transition-colors';

  const variantClasses = {
    primary: 'bg-blue-600 text-white hover:bg-blue-700',
    secondary: 'bg-gray-600 text-white hover:bg-gray-700',
    danger: 'bg-red-600 text-white hover:bg-red-700',
  };

  const sizeClasses = {
    sm: 'px-2 py-1 text-sm',
    md: 'px-4 py-2',
    lg: 'px-6 py-3 text-lg',
  };
</script>

<button class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]}" {disabled} {onclick}>
  {@render children()}
</button>
```

### 2. Store統合コンポーネント

```svelte
<!-- RepositoryCard.svelte -->
<script lang="ts">
  import { repositories, type Repository } from '../stores';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    repository: Repository;
  }

  let { repository }: Props = $props();

  // Runesで状態管理
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // 派生状態
  let isRunning = $derived(repository.mcpServer?.status === 'running');
  let statusColor = $derived(isRunning ? 'green' : 'gray');

  async function startServer() {
    isLoading = true;
    error = null;

    try {
      const result = await invoke('start_repository_mcp_server', {
        repositoryId: repository.id,
        repoPath: repository.path,
      });

      // Storeを更新
      repositories.update((repos) =>
        repos.map((r) =>
          r.id === repository.id ? { ...r, mcpServer: { port: 9500, status: 'running' } } : r
        )
      );
    } catch (err) {
      error = err as string;
    } finally {
      isLoading = false;
    }
  }

  // エフェクトでクリーンアップ
  $effect(() => {
    return () => {
      // コンポーネントアンマウント時のクリーンアップ
    };
  });
</script>

<div class="bg-white p-6 rounded-lg border">
  <div class="flex justify-between items-start mb-4">
    <div>
      <h3 class="text-lg font-medium">{repository.name}</h3>
      <p class="text-sm text-gray-600">{repository.path}</p>
    </div>
    <span class="bg-{statusColor}-100 text-{statusColor}-800 px-2 py-1 rounded text-sm">
      {isRunning ? '実行中' : '停止中'}
    </span>
  </div>

  {#if error}
    <div class="mb-4 p-3 bg-red-50 text-red-700 rounded">
      {error}
    </div>
  {/if}

  <div class="flex gap-2">
    <button
      onclick={startServer}
      disabled={isLoading || isRunning}
      class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
    >
      {isLoading ? '起動中...' : '開始'}
    </button>
  </div>
</div>
```

### 3. フォームコンポーネント

```svelte
<!-- RepositoryForm.svelte -->
<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { repositories } from '../stores';

  interface FormData {
    name: string;
    path: string;
  }

  let formData = $state<FormData>({
    name: '',
    path: '',
  });

  let errors = $state<Partial<FormData>>({});

  // バリデーション
  let isValid = $derived(
    formData.name.trim() !== '' && formData.path.trim() !== '' && Object.keys(errors).length === 0
  );

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'リポジトリディレクトリを選択',
      });

      if (selected) {
        formData.path = selected as string;
        // パスからリポジトリ名を推測
        if (!formData.name) {
          const pathParts = formData.path.split('/');
          formData.name = pathParts[pathParts.length - 1];
        }
      }
    } catch (error) {
      console.error('Directory selection failed:', error);
    }
  }

  function validateField(field: keyof FormData, value: string) {
    errors = { ...errors };
    delete errors[field];

    if (!value.trim()) {
      errors[field] = `${field}は必須です`;
    }
  }

  function handleSubmit() {
    if (!isValid) return;

    const newRepo = {
      id: crypto.randomUUID(),
      name: formData.name.trim(),
      path: formData.path.trim(),
      isActive: false,
      lastUpdated: new Date(),
    };

    repositories.update((repos) => [...repos, newRepo]);

    // フォームリセット
    formData = { name: '', path: '' };
    errors = {};
  }
</script>

<form onsubmit|preventDefault={handleSubmit} class="space-y-4">
  <div>
    <label for="name" class="block text-sm font-medium text-gray-700 mb-1"> リポジトリ名 </label>
    <input
      id="name"
      bind:value={formData.name}
      oninput={(e) => validateField('name', e.target.value)}
      placeholder="リポジトリ名を入力"
      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500"
      class:border-red-500={errors.name}
    />
    {#if errors.name}
      <p class="text-red-500 text-sm mt-1">{errors.name}</p>
    {/if}
  </div>

  <div>
    <label for="path" class="block text-sm font-medium text-gray-700 mb-1"> パス </label>
    <div class="flex gap-2">
      <input
        id="path"
        bind:value={formData.path}
        oninput={(e) => validateField('path', e.target.value)}
        placeholder="リポジトリのパスを入力"
        class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500"
        class:border-red-500={errors.path}
      />
      <button
        type="button"
        onclick={selectDirectory}
        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
      >
        📁 選択
      </button>
    </div>
    {#if errors.path}
      <p class="text-red-500 text-sm mt-1">{errors.path}</p>
    {/if}
  </div>

  <button
    type="submit"
    disabled={!isValid}
    class="w-full px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
  >
    リポジトリを追加
  </button>
</form>
```

## State管理パターン

### 1. Storeの定義 (stores.ts)

```typescript
import { writable, derived } from 'svelte/store';

// 基本的なstore
export const repositories = writable<Repository[]>([]);
export const currentView = writable<'home' | 'repositories' | 'prompts' | 'settings'>('home');

// 派生store
export const runningServers = derived(repositories, ($repositories) =>
  $repositories.filter((r) => r.mcpServer?.status === 'running')
);

// カスタムstore
function createNotificationStore() {
  const { subscribe, set, update } = writable<Notification | null>(null);

  return {
    subscribe,
    show: (notification: Notification) => {
      set(notification);
      setTimeout(() => set(null), 3000);
    },
    clear: () => set(null),
  };
}

export const notification = createNotificationStore();
```

### 2. Storeの使用

```svelte
<script lang="ts">
  import { repositories, notification } from '../stores';

  // Runesでstore値を取得
  let $repositories = $state();
  let $notification = $state();

  // Storeの購読
  $effect(() => {
    const unsubscribeRepos = repositories.subscribe((value) => {
      $repositories = value;
    });

    const unsubscribeNotif = notification.subscribe((value) => {
      $notification = value;
    });

    return () => {
      unsubscribeRepos();
      unsubscribeNotif();
    };
  });
</script>
```

## TailwindCSS統合

### 1. 動的クラス

```svelte
<script lang="ts">
  let variant = $state<'primary' | 'secondary'>('primary');

  // 安全な動的クラス設定
  const buttonClasses = {
    primary: 'bg-blue-600 hover:bg-blue-700 text-white',
    secondary: 'bg-gray-600 hover:bg-gray-700 text-white',
  };
</script>

<button class="px-4 py-2 rounded {buttonClasses[variant]}"> ボタン </button>
```

### 2. 条件付きクラス

```svelte
<script lang="ts">
  let isActive = $state(false);
  let isLoading = $state(false);
</script>

<button
  class="px-4 py-2 rounded transition-colors"
  class:bg-blue-600={isActive}
  class:bg-gray-600={!isActive}
  class:opacity-50={isLoading}
  class:cursor-not-allowed={isLoading}
>
  ボタン
</button>
```

## テストとデバッグ

### 1. コンポーネントテスト

```javascript
// Button.test.js
import { render, fireEvent } from '@testing-library/svelte';
import Button from './Button.svelte';

test('renders button with correct text', () => {
  const { getByRole } = render(Button, {
    props: { children: 'Click me' },
  });

  expect(getByRole('button')).toHaveTextContent('Click me');
});

test('calls onclick when clicked', async () => {
  let clicked = false;
  const { getByRole } = render(Button, {
    props: {
      onclick: () => {
        clicked = true;
      },
      children: 'Click me',
    },
  });

  await fireEvent.click(getByRole('button'));
  expect(clicked).toBe(true);
});
```

### 2. デバッグテクニック

```svelte
<script lang="ts">
  let debugData = $state({ count: 0 });

  // デバッグ用エフェクト
  $effect(() => {
    console.log('Debug data changed:', debugData);
  });

  // 開発環境でのみ表示
  let isDev = $derived(import.meta.env.DEV);
</script>

{#if isDev}
  <div class="fixed bottom-4 right-4 bg-black text-white p-2 rounded text-xs">
    Debug: {JSON.stringify(debugData)}
  </div>
{/if}
```

## ベストプラクティス

1. **Runesの適切な使用**: $state、$derived、$effectを目的に応じて使い分け
2. **TypeScriptの活用**: propsや状態に型注釈を付与
3. **コンポーネント分割**: 単一責任の原則でコンポーネントを小さく保つ
4. **アクセシビリティ**: ARIAラベルやキーボードナビゲーションを考慮
5. **パフォーマンス**: 不要な再レンダリングを避ける
6. **エラーハンドリング**: 適切なエラー境界とフォールバック
7. **テスト**: コンポーネントの動作をテストで保証
