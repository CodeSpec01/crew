<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import HostHub from "../components/HostHub.svelte";

  let currentView = "menu";
  let generatedRoomCode = "";
  let hostIpAddress = "";
  let hostPort = ""; // New variable
  let activePeers: any[] = [];
  let unlistenPeers: UnlistenFn;

  // guest state
  let guestWs: WebSocket | null = null;
  let guestStatus = "Connecting...";

  // ws states
  let ws: WebSocket;
  let connectionStatus = "Disconnected";

  onMount(async () => {
    // 1. Host Listener (Kept from before)
    unlistenPeers = await listen("peer-update", (event) => {
      activePeers = event.payload as any[];
    });

    // 2. Guest URL Parsing
    // When the phone loads the app, check if it was from a QR scan
    const urlParams = new URLSearchParams(window.location.search);
    const joinCode = urlParams.get("join");
    const targetPort = urlParams.get("port");

    if (joinCode && targetPort) {
      // We are a guest! Bypass the menu.
      currentView = "guest_session";
      // The hostname of the URL is the Host's IP (e.g., 192.168.1.133)
      const targetIp = window.location.hostname;
      connectAsGuest(targetIp, targetPort, joinCode);
    }
  });

  onDestroy(() => {
    if (unlistenPeers) unlistenPeers();
    if (guestWs) guestWs.close();
  });

  // The actual guest connection logic
  function connectAsGuest(ip: string, port: string, code: string) {
    guestWs = new WebSocket(`ws://${ip}:${port}/ws`);

    guestWs.onopen = () => {
      guestStatus = `Connected to Crew: ${code}`;
      const joinRequest = {
        type: "JOIN_REQUEST",
        // We will make this dynamic later, hardcoded for now to prove it works
        payload: { name: "MOBILE_NODE_1" },
      };
      guestWs?.send(JSON.stringify(joinRequest));
    };

    guestWs.onmessage = (event) => {
      console.log("Host says:", event.data);
    };

    guestWs.onclose = () => {
      guestStatus = "Disconnected from Host.";
      currentView = "menu"; // Kick them back to the menu
    };

    guestWs.onerror = () => {
      guestStatus = "Connection failed.";
    };
  }

  // Simulate a guest joining the room
  function testGuestConnection() {
    // Connect to the Rust WebSocket server we just built
    ws = new WebSocket(`ws://${hostIpAddress}:${hostPort}/ws`);

    ws.onopen = () => {
      connectionStatus = "Connected to Host!";
      // Send the JSON protocol message
      const joinRequest = {
        type: "JOIN_REQUEST",
        payload: { name: "DEV_STATION_ALPHA" },
      };
      ws.send(JSON.stringify(joinRequest));
    };

    ws.onmessage = (event) => {
      console.log("Message from Rust Server:", event.data);
      alert("Rust says: " + event.data); // Quick visual pop-up for testing
    };

    ws.onclose = () => {
      connectionStatus = "Disconnected";
    };
  }

  async function handleHostClick() {
    try {
      // Receive the combined string from Rust
      const response = await invoke<string>("start_host_session");

      // Split it into variables
      const [code, ip, port] = response.split("|");
      generatedRoomCode = code;
      hostIpAddress = ip;
      hostPort = port;

      currentView = "hosting";
    } catch (error) {
      console.error("Failed to start server:", error);
    }
  }

  async function handleCloseSession() {
    try {
      await invoke("stop_host_session"); // Tell Rust to kill the server
      currentView = "menu"; // Return UI to the main menu
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
    {#if currentView !== "menu"}
      <button
        on:click={handleCloseSession}
        class="text-text-secondary hover:text-text-primary"
      >
        <span class="material-symbols-outlined">close</span>
      </button>
    {/if}
  </header>

  <div class="flex-1 w-full">
    {#if currentView === "menu"}
      <div class="flex flex-col items-center justify-center h-full gap-6 mt-20">
        <p
          class="w-64 bg-primary-container text-background font-bold py-3 rounded uppercase tracking-wider hover:shadow-[0_0_15px_theme('colors.primary-container')] transition-all"
        >
          {hostIpAddress}
        </p>
        <button
          on:click={handleHostClick}
          class="w-64 bg-primary-container text-background font-bold py-3 rounded uppercase tracking-wider hover:shadow-[0_0_15px_theme('colors.primary-container')] transition-all"
        >
          Start a Crew
        </button>
        <button
          on:click={testGuestConnection}
          class="mt-4 text-primary-container underline"
        >
          [Test WS Guest Handshake]
        </button>
        <button
          on:click={() => (currentView = "guest")}
          class="w-64 bg-surface border border-border text-text-primary font-bold py-3 rounded uppercase tracking-wider hover:bg-border transition-all"
        >
          Join a Crew
        </button>
      </div>
    {:else if currentView === "hosting"}
      <HostHub
        hostIp={hostIpAddress}
        {hostPort}
        roomCode={generatedRoomCode}
        connectedPeers={activePeers}
      />
    {:else if currentView === "guest_session"}
      <div class="flex flex-col items-center justify-center h-full gap-4 mt-20">
        <div
          class="w-12 h-12 rounded-full border-t-2 border-primary-container animate-spin"
        ></div>
        <h2 class="font-mono text-xl text-primary-container">{guestStatus}</h2>
        <p class="text-text-secondary font-mono text-sm">
          Waiting for host commands...
        </p>
      </div>
    {/if}
  </div>
</div>
