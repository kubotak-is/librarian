<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { prompts, repositories, isLoading, notification } from '../stores';
  import type { Prompt } from '../stores';

  let searchQuery = '';
  let selectedCategory = 'all';
  let selectedPrompt: Prompt | null = null;
  let categories: string[] = [];

  // プロンプトをロード
  async function loadPrompts() {
    isLoading.set(true);

    try {
      const allPrompts: Prompt[] = [];
      const activeRepos = $repositories.filter((repo) => repo.isActive);

      for (const repo of activeRepos) {
        try {
          const result = await invoke('parse_agent_library', {
            repoPath: repo.path,
          });

          if (result && (result as any).prompts) {
            const repoPrompts = (result as any).prompts.map((p: any) => ({
              id: `${repo.id}_${p.id}`,
              name: p.id,
              title: p.title || p.id,
              description: p.description || '',
              category: p.category || 'その他',
              repository: repo.name,
              content: p.content,
            }));

            allPrompts.push(...repoPrompts);
          }
        } catch (error) {
          console.error(`Failed to load prompts from ${repo.name}:`, error);
        }
      }

      prompts.set(allPrompts);

      // カテゴリリストを更新
      const uniqueCategories = [...new Set(allPrompts.map((p) => p.category))];
      categories = ['all', ...uniqueCategories];
    } catch (error) {
      notification.set({
        type: 'error',
        message: `プロンプトの読み込みに失敗しました: ${error}`,
      });
    } finally {
      isLoading.set(false);
    }
  }

  // フィルタリング
  $: filteredPrompts = $prompts.filter((prompt) => {
    const matchesSearch =
      prompt.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      prompt.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesCategory = selectedCategory === 'all' || prompt.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  function showPromptDetail(prompt: Prompt) {
    selectedPrompt = prompt;
  }

  function closePromptDetail() {
    selectedPrompt = null;
  }

  // 初期ロード
  $: if ($repositories.length > 0) {
    loadPrompts();
  }
</script>

<div class="space-y-6">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold text-gray-900">プロンプト一覧</h2>
    <button
      onclick={loadPrompts}
      disabled={$isLoading}
      class="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
    >
      {$isLoading ? '読み込み中...' : '更新'}
    </button>
  </div>

  <!-- 検索・フィルタ -->
  <div class="flex gap-4">
    <input
      bind:value={searchQuery}
      placeholder="プロンプトを検索..."
      class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
    />
    <select
      bind:value={selectedCategory}
      class="px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
    >
      <option value="all">全カテゴリ</option>
      {#each categories.filter((c) => c !== 'all') as category}
        <option value={category}>{category}</option>
      {/each}
    </select>
  </div>

  <!-- プロンプト一覧 -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
    {#each filteredPrompts as prompt (prompt.id)}
      <div
        class="bg-white p-6 rounded-lg border border-gray-200 shadow-sm hover:shadow-md transition-shadow cursor-pointer"
        onclick={() => showPromptDetail(prompt)}
        onkeydown={(e) => e.key === 'Enter' && showPromptDetail(prompt)}
        role="button"
        tabindex="0"
      >
        <div class="space-y-3">
          <div>
            <h3 class="text-lg font-medium text-gray-900">{prompt.title}</h3>
            <p class="text-sm text-gray-500">{prompt.repository}</p>
          </div>

          <p class="text-gray-600 text-sm line-clamp-3">{prompt.description}</p>

          <div class="flex justify-between items-center">
            <span
              class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
            >
              {prompt.category}
            </span>
            <span class="text-xs text-gray-400">{prompt.name}</span>
          </div>
        </div>
      </div>
    {:else}
      <div class="col-span-full text-center py-12">
        <div class="text-gray-400 text-lg mb-2">📝</div>
        {#if $repositories.length === 0}
          <p class="text-gray-500">リポジトリが登録されていません</p>
          <p class="text-sm text-gray-400">まずリポジトリを追加してください</p>
        {:else if $repositories.filter((r) => r.isActive).length === 0}
          <p class="text-gray-500">有効なリポジトリがありません</p>
          <p class="text-sm text-gray-400">リポジトリを有効にしてください</p>
        {:else}
          <p class="text-gray-500">プロンプトが見つかりません</p>
          <p class="text-sm text-gray-400">検索条件を変更してみてください</p>
        {/if}
      </div>
    {/each}
  </div>
</div>

<!-- プロンプト詳細モーダル -->
{#if selectedPrompt}
  <div
    class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50"
    onclick={closePromptDetail}
    onkeydown={(e) => e.key === 'Escape' && closePromptDetail()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div
      class="relative top-20 mx-auto p-5 border w-11/12 md:w-3/4 lg:w-1/2 shadow-lg rounded-md bg-white"
    >
      <div class="space-y-4">
        <div class="flex justify-between items-start">
          <div>
            <h3 class="text-lg font-bold text-gray-900">{selectedPrompt.title}</h3>
            <p class="text-sm text-gray-500">
              {selectedPrompt.repository} / {selectedPrompt.category}
            </p>
          </div>
          <button onclick={closePromptDetail} class="text-gray-400 hover:text-gray-600 text-xl">
            ×
          </button>
        </div>

        <div class="border-t pt-4">
          <h4 class="font-medium text-gray-900 mb-2">説明</h4>
          <p class="text-gray-600">{selectedPrompt.description}</p>
        </div>

        {#if selectedPrompt.content}
          <div class="border-t pt-4">
            <h4 class="font-medium text-gray-900 mb-2">プロンプト内容</h4>
            <pre
              class="bg-gray-50 p-4 rounded-md text-sm overflow-x-auto whitespace-pre-wrap">{selectedPrompt.content}</pre>
          </div>
        {/if}

        <div class="border-t pt-4 flex justify-end">
          <button
            onclick={closePromptDetail}
            class="bg-gray-600 text-white px-4 py-2 rounded-md hover:bg-gray-700 transition-colors"
          >
            閉じる
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
