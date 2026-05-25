<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import HostHub from "../components/HostHub.svelte";

  let currentView = "menu";
  let generatedRoomCode = "";
  let hostIpAddress = "";

  async function handleHostClick() {
    try {
      // Receive the combined string from Rust
      const response = await invoke<string>("start_host_session");

      // Split it into variables
      const [code, ip] = response.split("|");
      generatedRoomCode = code;
      hostIpAddress = ip;

      currentView = "hosting";
    } catch (error) {
      console.error("Failed to start server:", error);
    }
  }

  async function handleCloseSession() {
    try {
      await invoke("stop_host_session"); // Tell Rust to kill the server
      currentView = 'menu';              // Return UI to the main menu
    } catch (error) {
      console.error("Failed to stop server:", error);
    }
  }
</script>

<div class="min-h-screen flex flex-col bg-background p-6">
  <header
    class="w-full flex justify-between items-center mb-8 max-w-3xl mx-auto"
  >
    <h1 class="font-mono text-xl font-bold text-primary-container">CREW</h1>
    {#if currentView !== 'menu'}
      <button on:click={handleCloseSession} class="text-text-secondary hover:text-text-primary">
        <span class="material-symbols-outlined">close</span>
      </button>
    {/if}
  </header>

  <div class="flex-1 w-full">
    {#if currentView === "menu"}
      <div class="flex flex-col items-center justify-center h-full gap-6 mt-20">
        <button
          on:click={handleHostClick}
          class="w-64 bg-primary-container text-background font-bold py-3 rounded uppercase tracking-wider hover:shadow-[0_0_15px_theme('colors.primary-container')] transition-all"
        >
          Start a Crew
        </button>
        <button
          on:click={() => (currentView = "guest")}
          class="w-64 bg-surface border border-border text-text-primary font-bold py-3 rounded uppercase tracking-wider hover:bg-border transition-all"
        >
          Join a Crew
        </button>
      </div>
    {:else if currentView === "hosting"}
      <HostHub hostIpAddress={hostIpAddress} roomCode={generatedRoomCode} />
    {/if}
  </div>
</div>
