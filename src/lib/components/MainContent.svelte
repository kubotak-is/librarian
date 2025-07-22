<script lang="ts">
  import PromptsView from './views/PromptsView.svelte';
  import SettingsView from './views/SettingsView.svelte';
  import EmptyState from './EmptyState.svelte';
  import type { RepositoryConfig } from '../stores/config';

  // Props: Layoutコンポーネントから受け取る
  let {
    selectedRepository = null,
    currentView = 'prompts',
  }: {
    selectedRepository: RepositoryConfig | null;
    currentView?: string;
  } = $props();
</script>

<!-- メインコンテンツエリア -->
<div class="h-full bg-primary">
  {#if selectedRepository}
    <div class="h-full fade-in">
      {#if currentView === 'prompts'}
        <PromptsView repository={selectedRepository} />
      {:else if currentView === 'settings'}
        <SettingsView repository={selectedRepository} />
      {/if}
    </div>
  {:else}
    <EmptyState />
  {/if}
</div>
