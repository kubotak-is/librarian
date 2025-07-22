<script lang="ts">
  import { IconFolder as Folder, IconPlus as Plus } from '@tabler/icons-svelte';
</script>

<!-- 空の状態 -->
<div class="h-full flex items-center justify-center bg-primary">
  <div class="text-center max-w-md px-8">
    <div class="mb-6">
      <Folder size={64} class="mx-auto text-secondary opacity-50" />
    </div>

    <h2 class="text-xl font-semibold text-primary mb-3">Librarianへようこそ</h2>

    <p class="text-secondary mb-6 leading-relaxed">
      AI agent prompts を管理するためのデスクトップアプリケーションです。<br />
      まずは .agent_library を含むリポジトリを追加してください。
    </p>

    <div class="space-y-4">
      <div class="bg-secondary rounded-lg p-4 text-left">
        <h3 class="font-medium text-primary mb-2">使い方</h3>
        <ol class="text-sm text-secondary space-y-1">
          <li>1. 左側のサイドバーで「リポジトリを追加」をクリック</li>
          <li>2. .agent_library ディレクトリを含むフォルダを選択</li>
          <li>3. MCPサーバーを起動してプロンプトを利用開始</li>
        </ol>
      </div>

      <div class="flex items-center justify-center space-x-2 text-accent">
        <Plus size={16} />
        <span class="text-sm">左サイドバーから開始</span>
      </div>
    </div>
  </div>
</div>
