# UI/UX改善ガイド

> Librarianアプリケーションのユーザーインターフェースとユーザーエクスペリエンスを向上させるためのガイドライン

## 概要

このガイドは、Librarian（Tauri + Svelte 5 + TailwindCSS）アプリケーションのUI/UX改善において、レスポンシブデザイン、アクセシビリティ、ユーザビリティを向上させる方法を提供します。

## レスポンシブデザインの実装

### 基本原則

1. **モバイルファースト設計**
   - 最小画面幅から設計開始
   - 段階的にデスクトップサイズに拡張

2. **適切なブレークポイント**
   ```css
   /* TailwindCSS標準ブレークポイント */
   sm: 640px   /* タブレット */
   md: 768px   /* 小型デスクトップ */
   lg: 1024px  /* デスクトップ */
   xl: 1280px  /* 大型デスクトップ */
   ```

### レスポンシブレイアウトパターン

#### 1. 編集ボタンの見切れ問題解決

**問題**: 画面幅が狭いとき、編集ボタンが画面外に出てしまう

**解決策**:

```svelte
<!-- タイトルとボタンを分離したレイアウト -->
<div class="mb-4">
  <h1 class="text-xl font-semibold text-gray-100 mb-2">
    {title}
  </h1>

  <!-- 編集ボタンを独立した行に配置 -->
  <div class="flex justify-end">
    <button class="flex items-center gap-1.5 px-3 py-1.5">
      <Edit size={14} />
      <span class="hidden sm:inline">編集</span>
    </button>
  </div>
</div>
```

**キーポイント**:

- `justify-end`でボタンを右端配置
- `hidden sm:inline`で小画面ではテキスト非表示
- ボタンは独立した行で視認性確保

#### 2. コンテンツ表示の最適化

**横スクロール対応**:

```svelte
<!-- プロンプト内容の表示 -->
<div class="h-full overflow-x-auto overflow-y-auto">
  <pre
    class="p-4 font-mono text-sm text-gray-100 leading-relaxed whitespace-pre"
    style="tab-size: 2; min-width: 100%;">{content}</pre>
</div>
```

**重要事項**:

- `overflow-x-auto`: 水平スクロール有効
- `whitespace-pre`: 改行とスペースを保持
- `min-width: 100%`: 最小幅を保証

## アクセシビリティの改善

### キーボードナビゲーション

```svelte
<div
  role="button"
  tabindex="0"
  onclick={() => selectItem(item)}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      selectItem(item);
    }
  }}
>
  {item.title}
</div>
```

### ARIAラベルとタイトル

```svelte
<button title="プロンプトを編集" aria-label="プロンプト編集ボタン" class="...">
  <Edit size={14} />
  <span class="sr-only">編集</span>
</button>
```

## パフォーマンス最適化

### 仮想スクロール（大量データ対応）

```svelte
<script>
  import { derived } from 'svelte/store';

  let containerHeight = 400;
  let itemHeight = 60;
  let scrollTop = 0;

  // 表示範囲の計算
  $: visibleStart = Math.floor(scrollTop / itemHeight);
  $: visibleEnd = Math.min(
    visibleStart + Math.ceil(containerHeight / itemHeight) + 1,
    items.length
  );
  $: visibleItems = items.slice(visibleStart, visibleEnd);
</script>

<div class="overflow-auto" style="height: {containerHeight}px" bind:scrollTop>
  <!-- スクロールスペーサー -->
  <div style="height: {visibleStart * itemHeight}px"></div>

  {#each visibleItems as item}
    <div style="height: {itemHeight}px">
      <!-- アイテム内容 -->
    </div>
  {/each}

  <!-- 下部スペーサー -->
  <div style="height: {(items.length - visibleEnd) * itemHeight}px"></div>
</div>
```

## ユーザビリティの向上

### 状態表示の改善

```svelte
<!-- ローディング状態 -->
{#if loading}
  <div class="flex items-center justify-center p-4">
    <div
      class="animate-spin w-6 h-6 border-2 border-accent-600 border-t-transparent rounded-full"
    ></div>
    <span class="ml-2 text-gray-400">読み込み中...</span>
  </div>
{/if}

<!-- エラー状態 -->
{#if error}
  <div class="p-4 bg-red-900/50 border border-red-500 rounded-md">
    <p class="text-red-200">{error}</p>
    <button onclick={retry} class="mt-2 text-red-300 underline"> 再試行 </button>
  </div>
{/if}
```

### フィードバックの強化

```svelte
<!-- 成功時の通知 -->
{#if showSuccess}
  <div
    class="fixed top-4 right-4 bg-green-600 text-white px-4 py-2 rounded-md shadow-lg transition-all duration-300"
    transition:fly={{ x: 100, duration: 300 }}
  >
    <div class="flex items-center gap-2">
      <Check size={16} />
      <span>保存しました</span>
    </div>
  </div>
{/if}
```

## 一般的な問題と解決策

### 1. テキストオーバーフロー

**問題**: 長いテキストがレイアウトを崩す

**解決策**:

```css
/* 単語での折り返し */
.text-content {
  @apply break-words overflow-wrap-anywhere;
}

/* 省略表示 */
.text-ellipsis {
  @apply truncate;
}
```

### 2. 固定要素の配置問題

**問題**: 固定要素（ボタン、メニュー）が他の要素と重複

**解決策**:

```svelte
<div class="relative">
  <!-- メインコンテンツ -->
  <div class="pr-20">
    {content}
  </div>

  <!-- 固定ボタン -->
  <div class="absolute top-0 right-0">
    <button>編集</button>
  </div>
</div>
```

### 3. フォーカス管理

**問題**: キーボード操作時のフォーカス状態が不明確

**解決策**:

```css
/* フォーカスの可視化 */
.focus-visible {
  @apply outline-2 outline-offset-2 outline-accent-500;
}

/* フォーカストラップ */
.focus-trap {
  @apply focus-within:ring-2 focus-within:ring-accent-500;
}
```

## テスト方針

### レスポンシブテスト

```javascript
// Playwright E2Eテスト
test('編集ボタンは全画面サイズで表示される', async ({ page }) => {
  const viewports = [
    { width: 375, height: 667 }, // Mobile
    { width: 768, height: 1024 }, // Tablet
    { width: 1920, height: 1080 }, // Desktop
  ];

  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    const editButton = page.locator('[data-testid="edit-button"]');
    await expect(editButton).toBeVisible();
  }
});
```

### アクセシビリティテスト

```javascript
test('キーボードナビゲーションが動作する', async ({ page }) => {
  await page.keyboard.press('Tab');
  const focusedElement = await page.locator(':focus');
  await expect(focusedElement).toBeVisible();

  await page.keyboard.press('Enter');
  // 期待される動作の確認
});
```

## まとめ

UI/UX改善は継続的なプロセスです：

1. **ユーザーフィードバックの収集**
2. **実際の使用パターンの分析**
3. **段階的な改善の実装**
4. **テストによる品質保証**

重要なのは、技術的な実装だけでなく、実際のユーザビリティを考慮した設計を行うことです。
