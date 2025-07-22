<script lang="ts">
  import {
    IconFolder as Folder,
    IconPlus as Plus,
    IconPlayerPlay as Play,
  } from '@tabler/icons-svelte';
  import type { RepositoryConfig } from '../stores/config';

  // Props: Layoutコンポーネントから受け取る
  let {
    repositories,
    selectedRepository,
    onRepositorySelect,
    onAddRepository,
  }: {
    repositories: RepositoryConfig[];
    selectedRepository: RepositoryConfig | null;
    // eslint-disable-next-line no-unused-vars
    onRepositorySelect: (repository: RepositoryConfig) => Promise<void>;
    onAddRepository?: () => Promise<void>;
  } = $props();

  // リポジトリを選択
  async function selectRepository(repository: RepositoryConfig) {
    // 親コンポーネント（Layout）のハンドラーを使用
    await onRepositorySelect(repository);
  }

  // リポジトリ追加処理
  async function handleAddRepository() {
    if (onAddRepository) {
      await onAddRepository();
    }
  }
</script>

<!-- サイドバー -->
<div class="h-full flex flex-col">
  <!-- ヘッダー -->
  <div class="p-4 border-b border-gray-600/30">
    <h2 class="text-sm font-semibold text-gray-100">リポジトリ</h2>
  </div>

  <!-- リポジトリ一覧 -->
  <div class="flex-1 overflow-y-auto p-2">
    {#each repositories as repository}
      <div
        class="flex items-center px-4 py-3 mx-2 text-sm rounded-xl cursor-pointer transition-all duration-200 ease-out {selectedRepository?.id ===
        repository.id
          ? 'bg-accent-600 text-white shadow-lg shadow-accent-600/20'
          : 'text-gray-400 hover:bg-gray-700/60 hover:text-gray-100'}"
        style="margin: 4px 0;"
        role="button"
        tabindex="0"
        onclick={() => selectRepository(repository)}
        onkeydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            selectRepository(repository);
          }
        }}
      >
        <div class="flex items-center flex-1 min-w-0">
          {#if repository.mcp_server && repository.mcp_server.status === 'running'}
            <Play size={16} class="mr-2 flex-shrink-0 text-green-500" />
          {:else}
            <Folder size={16} class="mr-2 flex-shrink-0 text-gray-400" />
          {/if}
          <span class="flex-1 truncate text-sm">{repository.name}</span>
        </div>

        <!-- MCP サーバー状態インジケーター -->
        {#if repository.mcp_server}
          <div
            class="status-indicator status-{repository.mcp_server.status} ml-2"
            title="MCP Server: {repository.mcp_server.status}"
          ></div>
        {/if}
      </div>
    {/each}

    <!-- リポジトリが無い場合のメッセージ -->
    {#if repositories.length === 0}
      <div class="p-4 text-center text-gray-400">
        <Folder size={32} class="mx-auto mb-2 opacity-50" />
        <p class="text-sm">リポジトリがありません</p>
        <p class="text-xs mt-1">下のボタンから追加してください</p>
      </div>
    {/if}
  </div>

  <!-- 追加ボタン -->
  <div class="border-t border-gray-600/30">
    <button
      class="w-full flex items-center justify-center space-x-2 px-4 py-4 text-sm font-medium text-gray-400 hover:text-gray-100 hover:bg-gray-700/60 transition-colors duration-200"
      onclick={handleAddRepository}
      title="新しいリポジトリを追加"
    >
      <Plus size={18} />
      <span>リポジトリを追加</span>
    </button>
  </div>
</div>

<style>
  .repo-item {
    position: relative;
  }
</style>
