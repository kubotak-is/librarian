<script lang="ts">
  import { currentView } from '../stores';

  function navigateTo(view: 'home' | 'repositories' | 'prompts' | 'settings') {
    currentView.set(view);
  }
</script>

<nav class="bg-white shadow-sm border-b border-gray-200">
  <div class="max-w-7xl mx-auto px-4">
    <div class="flex justify-between h-16">
      <div class="flex items-center">
        <h1 class="text-xl font-bold text-gray-900">📚 librarian</h1>
      </div>

      <div class="flex items-center space-x-8">
        <button
          onclick={() => navigateTo('home')}
          class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium transition-colors"
        >
          ホーム
        </button>
        <button
          onclick={() => navigateTo('repositories')}
          class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium transition-colors"
        >
          リポジトリ
        </button>
        <button
          onclick={() => navigateTo('prompts')}
          class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium transition-colors"
        >
          プロンプト
        </button>
        <button
          onclick={() => navigateTo('settings')}
          class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium transition-colors"
        >
          接続ガイド
        </button>
      </div>
    </div>
  </div>
</nav>
