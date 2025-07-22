<script lang="ts">
  import { notification } from '../stores';
  import { onMount } from 'svelte';

  let timeoutId: number;

  onMount(() => {
    const unsubscribe = notification.subscribe((value) => {
      if (value) {
        // 3秒後に自動で消す
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
          notification.set(null);
        }, 3000);
      }
    });

    return unsubscribe;
  });

  function closeNotification() {
    notification.set(null);
    clearTimeout(timeoutId);
  }
</script>

{#if $notification}
  <div class="fixed top-4 right-4 z-50 max-w-sm w-full">
    <div
      class="bg-white border border-gray-200 rounded-lg shadow-lg p-4 {$notification.type ===
      'success'
        ? 'border-l-4 border-l-green-500'
        : $notification.type === 'error'
          ? 'border-l-4 border-l-red-500'
          : 'border-l-4 border-l-blue-500'}"
    >
      <div class="flex items-start">
        <div class="flex-shrink-0">
          {#if $notification.type === 'success'}
            <div class="text-green-500">✅</div>
          {:else if $notification.type === 'error'}
            <div class="text-red-500">❌</div>
          {:else}
            <div class="text-blue-500">ℹ️</div>
          {/if}
        </div>
        <div class="ml-3 flex-1">
          <p class="text-sm text-gray-900">{$notification.message}</p>
        </div>
        <div class="ml-4 flex-shrink-0">
          <button onclick={closeNotification} class="text-gray-400 hover:text-gray-600">
            <span class="sr-only">Close</span>
            ×
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
