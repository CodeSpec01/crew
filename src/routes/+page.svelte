<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import HostHub from '../components/HostHub.svelte';

  let currentView = 'menu';
  let generatedRoomCode = '';

  async function handleHostClick() {
    generatedRoomCode = await invoke("start_host_session");
    currentView = 'hosting';
  }
</script>

<div class="min-h-screen flex flex-col bg-background p-6">
  
  <header class="w-full flex justify-between items-center mb-8 max-w-3xl mx-auto">
    <h1 class="font-mono text-xl font-bold text-primary-container">CREW</h1>
    {#if currentView !== 'menu'}
      <button on:click={() => currentView = 'menu'} class="text-text-secondary hover:text-text-primary">
        <span class="material-symbols-outlined">close</span>
      </button>
    {/if}
  </header>

  <div class="flex-1 w-full">
    {#if currentView === 'menu'}
      <div class="flex flex-col items-center justify-center h-full gap-6 mt-20">
        <button on:click={handleHostClick} class="w-64 bg-primary-container text-background font-bold py-3 rounded uppercase tracking-wider hover:shadow-[0_0_15px_theme('colors.primary-container')] transition-all">
          Start a Crew
        </button>
        <button on:click={() => currentView = 'guest'} class="w-64 bg-surface border border-border text-text-primary font-bold py-3 rounded uppercase tracking-wider hover:bg-border transition-all">
          Join a Crew
        </button>
      </div>
    {:else if currentView === 'hosting'}
      <HostHub roomCode={generatedRoomCode} />
    {/if}
  </div>
</div>