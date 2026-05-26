<script lang="ts">
  import QRCode from 'qrcode';

  export let roomCode: string = "----";
  export let hostIp: string = "127.0.0.1";
  export let hostPort: string = "6769";
  export let connectedPeers: any[] = [];

  let qrDataUrl = "";

  // Svelte's reactive block: This runs automatically whenever roomCode or hostIp changes
  $: {
    if (roomCode !== "----" && hostIp !== "127.0.0.1") {
      // 1. Point to Vite (1420) for the UI in development
      // 2. Pass both the room code AND the Axum port as URL parameters
      const joinUrl = `http://${hostIp}:1420/?join=${roomCode}&port=${hostPort}`;
      
      QRCode.toDataURL(joinUrl, {
        color: {
          dark: '#00F0FF',
          light: '#090A0F'
        },
        margin: 2,
        width: 200
      }).then(url => {
        qrDataUrl = url;
      }).catch(err => {
        console.error("QR Error:", err);
      });
    }
  }
</script>

<main class="w-full max-w-3xl mx-auto flex flex-col gap-8 pb-24">
  
  <div class="flex items-center justify-center gap-2 mt-4">
    <div class="w-2 h-2 rounded-full bg-primary-container animate-pulse shadow-[0_0_8px_theme('colors.primary-container')]"></div>
    <span class="font-mono text-xs text-primary-container uppercase tracking-widest">Broadcasting on Local Net</span>
  </div>

  <div class="flex flex-col md:flex-row items-center justify-center gap-10 mt-4">
    
    <div class="bg-surface backdrop-blur-2xl border border-border p-6 rounded-xl relative">
      <div class="absolute top-0 left-0 w-4 h-4 border-t-2 border-l-2 border-primary-container rounded-tl-xl m-2 opacity-50"></div>
      <div class="absolute top-0 right-0 w-4 h-4 border-t-2 border-r-2 border-primary-container rounded-tr-xl m-2 opacity-50"></div>
      <div class="absolute bottom-0 left-0 w-4 h-4 border-b-2 border-l-2 border-primary-container rounded-bl-xl m-2 opacity-50"></div>
      <div class="absolute bottom-0 right-0 w-4 h-4 border-b-2 border-r-2 border-primary-container rounded-br-xl m-2 opacity-50"></div>
      
      {#if qrDataUrl}
        <img src={qrDataUrl} alt="Join QR Code" class="w-[200px] h-[200px] border border-border/50 rounded" />
      {:else}
        <div class="w-[200px] h-[200px] border border-border/50 flex items-center justify-center bg-background/50 text-text-secondary font-mono text-sm">
          GENERATING...
        </div>
      {/if}
    </div>

    <div class="flex flex-col items-center gap-2">
      <span class="font-mono text-sm text-text-secondary uppercase tracking-wider">Shout Code</span>
      <span class="font-mono text-[48px] leading-none font-bold text-primary-container tracking-[0.2em]">{roomCode}</span>
      <span class="font-mono text-[48px] leading-none font-bold text-primary-container tracking-[0.2em]">{hostIp}</span>
    </div>
  </div>

  <div class="mt-4 flex flex-col gap-3">
    <h2 class="font-mono text-sm text-text-secondary uppercase border-b border-border pb-2 mb-2">
      Connected Crew ({connectedPeers.length})
    </h2>
    
    {#each connectedPeers as peer}
      <div class="bg-surface/40 backdrop-blur-xl border border-border p-3 rounded flex items-center justify-between hover:bg-surface/60 transition-colors">
        <div class="flex items-center gap-4">
          <span class="material-symbols-outlined text-secondary">{peer.type}</span>
          <div class="flex flex-col">
            <span class="font-mono text-sm text-text-primary font-bold">{peer.name}</span>
            <span class="font-mono text-xs text-text-secondary">{peer.ip}</span>
          </div>
        </div>
        <button class="text-text-secondary hover:text-error transition-colors p-1 rounded hover:bg-error/10">
          <span class="material-symbols-outlined text-[18px]">close</span>
        </button>
      </div>
    {/each}

    {#if connectedPeers.length === 0}
      <div class="text-center p-6 border border-border border-dashed rounded text-text-secondary font-mono text-sm">
        Waiting for peers to join...
      </div>
    {/if}
  </div>
</main>