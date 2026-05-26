<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  let logs: string[] = ["UI Initialized. Waiting for Rust..."];
  let unlistenLog: UnlistenFn;

  onMount(async () => {
    // Listen for events coming from the Axum server
    unlistenLog = await listen<string>('server-log', (event) => {
      // Add the new log to the array, keeping only the last 20 to prevent lag
      logs = [...logs, event.payload].slice(-20);
    });
  });

  onDestroy(() => {
    if (unlistenLog) unlistenLog();
  });
</script>

<div class="min-h-screen bg-[#090A0F] text-[#00F0FF] p-6 font-mono flex flex-col gap-4">
  <h1 class="text-xl font-bold tracking-widest border-b border-[#00F0FF]/30 pb-2">
    CREW // DEV CONSOLE
  </h1>

  <div class="flex-1 bg-[#00F0FF]/5 border border-[#00F0FF]/30 rounded p-4 overflow-y-auto max-h-[70vh] flex flex-col justify-end">
    {#each logs as log}
      <div class="text-sm py-1 border-b border-[#00F0FF]/10 last:border-0">
        > {log}
      </div>
    {/each}
  </div>
</div>