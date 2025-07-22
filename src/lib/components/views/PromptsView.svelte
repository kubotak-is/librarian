<script lang="ts">
  import { onMount } from 'svelte';
  import {
    IconList as List,
    IconFileText as FileText,
    IconSearch as Search,
    IconEdit as Edit,
    IconCheck as Check,
    IconX as X,
    IconEye as Eye,
  } from '@tabler/icons-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { notification } from '../../stores';
  import type { RepositoryConfig } from '../../stores/config';

  let { repository }: { repository: RepositoryConfig } = $props();

  let prompts = $state<any[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let debouncedSearchQuery = $state('');
  let selectedPrompt = $state<any>(null);
  let isEditing = $state(false);
  let editContent = $state('');
  let saving = $state(false);
  let searchTimeout: number | null = null;

  // デバウンス付き検索クエリ更新
  function updateDebouncedSearch(query: string) {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    searchTimeout = setTimeout(() => {
      debouncedSearchQuery = query;
    }, 300);
  }

  // 検索クエリの変更を監視
  $effect(() => {
    updateDebouncedSearch(searchQuery);
  });

  // フィルタリングされたプロンプト（デバウンス済みクエリを使用）
  let filteredPrompts = $derived(
    prompts.filter(
      (prompt) =>
        prompt.title.toLowerCase().includes(debouncedSearchQuery.toLowerCase()) ||
        prompt.description.toLowerCase().includes(debouncedSearchQuery.toLowerCase())
    )
  );

  onMount(() => {
    loadPrompts();
  });

  // リポジトリが変更されたらプロンプトを再読み込み
  $effect(() => {
    if (repository) {
      // 編集モードを終了し、選択プロンプトをリセット
      isEditing = false;
      selectedPrompt = null;
      editContent = '';
      loadPrompts();
    }
  });

  async function loadPrompts() {
    loading = true;
    error = null;

    try {
      const agentLibrary = await invoke('parse_agent_library', {
        repoPath: repository.path,
      });

      prompts = (agentLibrary as any)?.prompts || [];

      // デフォルトで最初のプロンプトを選択（リポジトリ変更後は必ず最初のプロンプトを選択）
      if (prompts.length > 0) {
        selectedPrompt = prompts[0];
      } else {
        selectedPrompt = null;
      }
    } catch (err) {
      error = `プロンプトの読み込みに失敗しました: ${err}`;
      console.error('Failed to load prompts:', err);
    } finally {
      loading = false;
    }
  }

  function selectPrompt(prompt: any) {
    // 編集モードの場合、変更を保存するか確認
    if (isEditing && editContent !== selectedPrompt?.content) {
      const shouldSave = confirm('編集中の内容があります。保存しますか？');
      if (shouldSave) {
        // 保存処理は別途実装
        // 今回は簡単にキャンセル
      }
    }

    // 編集モードをキャンセル
    isEditing = false;
    editContent = '';

    selectedPrompt = prompt;
  }

  function formatDescription(description: string): string {
    return description.length > 100 ? description.substring(0, 100) + '...' : description;
  }

  function startEditing() {
    if (!selectedPrompt) return;
    isEditing = true;
    editContent = selectedPrompt.content || '';
  }

  function cancelEditing() {
    isEditing = false;
    editContent = '';
  }

  async function savePrompt() {
    if (!selectedPrompt || !repository) return;

    saving = true;
    try {
      // Tauriバックエンドでファイル保存
      await invoke('save_prompt_file', {
        repoPath: repository.path,
        promptId: selectedPrompt.id,
        content: editContent,
      });

      // ローカル状態を更新
      selectedPrompt.content = editContent;

      // プロンプト一覧を再読み込み
      await loadPrompts();

      isEditing = false;
      editContent = '';

      notification.set({
        type: 'success',
        message: 'プロンプトを保存しました',
      });
    } catch (err) {
      console.error('Failed to save prompt:', err);
      notification.set({
        type: 'error',
        message: `プロンプトの保存に失敗しました: ${err}`,
      });
    } finally {
      saving = false;
    }
  }
</script>

<!-- プロンプト一覧ビュー -->
<div class="h-full flex">
  <!-- 左側: プロンプト一覧 -->
  <div
    class="flex-shrink-0 border-r border-gray-600/30 flex flex-col bg-gray-700 w-80 min-w-64 max-w-96 lg:w-80"
  >
    <!-- ヘッダー -->
    <div class="p-4 border-b border-gray-600/30">
      <div class="flex items-center space-x-2 mb-3">
        <List size={16} class="text-gray-400" />
        <h2 class="text-sm font-semibold text-gray-100">プロンプト</h2>
        <span class="text-xs text-gray-400 bg-gray-800 px-2 py-1 rounded">
          {filteredPrompts.length}
        </span>
      </div>

      <!-- 検索バー -->
      <div class="relative">
        <Search
          size={16}
          class="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400"
        />
        <input
          type="text"
          placeholder="プロンプトを検索..."
          class="w-full pl-10 px-3 py-2 text-sm rounded-md bg-gray-800 text-gray-100 border border-gray-600/50 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-accent-500 focus:border-accent-500"
          bind:value={searchQuery}
        />
      </div>
    </div>

    <!-- プロンプト一覧 -->
    <div class="flex-1 overflow-y-auto">
      {#if loading}
        <div class="p-4 text-center text-gray-400">
          <div
            class="animate-spin w-6 h-6 border-2 border-accent-600 border-t-transparent rounded-full mx-auto mb-2"
          ></div>
          読み込み中...
        </div>
      {:else if error}
        <div class="p-4 text-center text-red-500">
          <p class="text-sm">{error}</p>
        </div>
      {:else if filteredPrompts.length === 0}
        <div class="p-4 text-center text-gray-400">
          {#if searchQuery}
            <p class="text-sm">検索結果が見つかりません</p>
          {:else}
            <FileText size={32} class="mx-auto mb-2 opacity-50" />
            <p class="text-sm">プロンプトがありません</p>
          {/if}
        </div>
      {:else}
        {#each filteredPrompts as prompt (prompt.id)}
          <div
            class="p-4 border-b border-gray-600/30 cursor-pointer transition-colors duration-150 {selectedPrompt?.id ===
            prompt.id
              ? 'prompt-selected'
              : 'hover:bg-gray-600/30'}"
            role="button"
            tabindex="0"
            onclick={() => selectPrompt(prompt)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                selectPrompt(prompt);
              }
            }}
          >
            <h3
              class="font-medium text-sm mb-1 {selectedPrompt?.id === prompt.id
                ? 'prompt-selected-title'
                : 'text-gray-100'}"
            >
              {prompt.title}
            </h3>
            <p
              class="text-xs {selectedPrompt?.id === prompt.id
                ? 'prompt-selected-desc'
                : 'text-gray-400'} leading-relaxed break-words"
            >
              {formatDescription(prompt.description)}
            </p>
            {#if prompt.category}
              <div class="mt-2">
                <span
                  class="inline-block text-xs px-2 py-1 rounded {selectedPrompt?.id === prompt.id
                    ? 'prompt-selected-category'
                    : 'bg-gray-800 text-gray-400'}"
                >
                  {prompt.category}
                </span>
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- 右側: プロンプト詳細 -->
  <div class="flex-1 flex flex-col bg-gray-800 min-w-0 overflow-hidden">
    {#if selectedPrompt}
      <!-- プロンプト詳細ヘッダー -->
      <div class="p-4 sm:p-6 border-b border-gray-600/30 flex-shrink-0 overflow-hidden">
        <!-- タイトル行 -->
        <div class="mb-4 w-full overflow-hidden">
          <h1
            class="text-xl font-semibold text-gray-100 mb-2 break-words overflow-wrap-anywhere max-w-full"
          >
            {selectedPrompt.title}
          </h1>

          <!-- 編集ボタンを独立した行に配置 -->
          <div class="flex justify-end">
            {#if isEditing}
              <div class="flex items-center gap-2">
                <button
                  class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md bg-green-600 text-white hover:bg-green-700 transition-colors duration-150 {saving
                    ? 'opacity-50 cursor-not-allowed'
                    : ''}"
                  onclick={savePrompt}
                  disabled={saving}
                  title="保存"
                >
                  <Check size={14} />
                  <span class="hidden sm:inline">{saving ? '保存中...' : '保存'}</span>
                </button>
                <button
                  class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md bg-gray-600 text-white hover:bg-gray-700 transition-colors duration-150"
                  onclick={cancelEditing}
                  disabled={saving}
                  title="キャンセル"
                >
                  <X size={14} />
                  <span class="hidden sm:inline">キャンセル</span>
                </button>
              </div>
            {:else}
              <button
                class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md bg-accent-600 text-white hover:bg-accent-700 transition-colors duration-150"
                onclick={startEditing}
                title="編集"
              >
                <Edit size={14} />
                <span class="hidden sm:inline">編集</span>
              </button>
            {/if}
          </div>
        </div>

        <!-- 説明とメタ情報 -->
        <div class="space-y-3 w-full overflow-hidden">
          <p class="text-gray-400 leading-relaxed break-words overflow-wrap-anywhere max-w-full">
            {selectedPrompt.description}
          </p>
          <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-gray-400">
            {#if selectedPrompt.category}
              <span class="bg-gray-700 px-3 py-1 rounded-full">
                {selectedPrompt.category}
              </span>
            {/if}
            <span>ID: {selectedPrompt.id}</span>
          </div>
        </div>
      </div>

      <!-- プロンプト内容 -->
      <div class="flex-1 flex flex-col overflow-hidden p-4 sm:p-6">
        <div class="bg-gray-700 rounded-lg overflow-hidden flex flex-col h-full w-full">
          <div
            class="flex items-center justify-between p-4 border-b border-gray-600/30 flex-shrink-0"
          >
            <h3 class="text-sm font-medium text-gray-100">プロンプト内容</h3>
            {#if isEditing}
              <div class="flex items-center space-x-2 text-xs text-gray-400">
                <Eye size={14} />
                <span>編集モード</span>
              </div>
            {/if}
          </div>

          <div class="flex-1 overflow-hidden w-full">
            {#if isEditing}
              <!-- 編集モード: テキストエリア -->
              <textarea
                bind:value={editContent}
                class="w-full h-full bg-gray-800 border-0 p-4 font-mono text-sm text-gray-100 resize-none focus:outline-none focus:ring-0 leading-relaxed"
                placeholder="プロンプト内容を入力してください..."
                disabled={saving}
                style="word-wrap: break-word; overflow-wrap: break-word; min-height: 400px; max-width: 100%;"
              ></textarea>
            {:else}
              <!-- 表示モード: 幅制限付き横スクロール -->
              <div class="h-full w-full overflow-hidden">
                <div class="h-full overflow-x-auto overflow-y-auto">
                  <pre
                    class="p-4 font-mono text-sm text-gray-100 leading-relaxed whitespace-pre break-all"
                    style="tab-size: 2; max-width: 100%; word-break: break-all; overflow-wrap: anywhere;">{selectedPrompt.content}</pre>
                </div>
              </div>
            {/if}
          </div>

          {#if isEditing}
            <div class="p-4 border-t border-gray-600/30 text-xs text-gray-400">
              Markdownフォーマットで記述してください
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <!-- プロンプト未選択状態 -->
      <div class="flex-1 flex items-center justify-center text-center">
        <div>
          <FileText size={48} class="mx-auto mb-4 text-gray-400 opacity-50" />
          <h3 class="text-lg font-medium text-gray-100 mb-2">プロンプトを選択</h3>
          <p class="text-gray-400">左側の一覧からプロンプトを選択してください</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* 選択状態のスタイル */
  .prompt-selected {
    background-color: #2563eb !important;
    color: white !important;
  }

  .prompt-selected-title {
    color: white !important;
    font-weight: 600 !important;
  }

  .prompt-selected-desc {
    color: rgba(255, 255, 255, 0.9) !important;
  }

  .prompt-selected-category {
    background-color: rgba(255, 255, 255, 0.2) !important;
    color: white !important;
  }
</style>
