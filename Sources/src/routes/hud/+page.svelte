<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let mode = $state<'VI' | 'EN'>('VI');
  let visible = $state(false);
  let fadeTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    const appWindow = getCurrentWindow();
    appWindow.setIgnoreCursorEvents(true);

    let unlisten: (() => void) | undefined;
    listen<string>('hud-update', (event) => {
      mode = (event.payload as 'VI' | 'EN');
      visible = true;

      if (fadeTimer) clearTimeout(fadeTimer);
      fadeTimer = setTimeout(() => {
        visible = false;
      }, 1200);
    }).then(fn => { unlisten = fn; });

    return () => {
      if (unlisten) unlisten();
    };
  });
</script>

<svelte:head>
  <title>VNKey HUD</title>
</svelte:head>

<div class="hud-root" class:visible>
  <div class="pill" class:vi={mode === 'VI'} class:en={mode === 'EN'}>
    <div class="flag-dot">
      {#if mode === 'VI'}
        <span class="flag-vi">🇻🇳</span>
      {:else}
        <span class="flag-en">🇺🇸</span>
      {/if}
    </div>
    <span class="mode-label">{mode}</span>
  </div>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(html, body) {
    background: transparent !important;
    overflow: hidden;
    width: 140px;
    height: 56px;
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', sans-serif;
  }

  .hud-root {
    width: 140px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transform: translateY(4px) scale(0.92);
    /* Single transition for both show and hide */
    transition: opacity 0.6s ease-out, transform 0.6s ease-out;
    pointer-events: none;
  }

  .hud-root.visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    transition: opacity 0.12s ease-out, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .pill {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    border-radius: 28px;
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.18);
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.35),
      0 2px 8px rgba(0, 0, 0, 0.25),
      inset 0 1px 0 rgba(255, 255, 255, 0.25);
    transform: scale(1);
    transition: transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
    will-change: transform, opacity;
  }

  .hud-root.visible .pill {
    transform: scale(1);
  }

  /* Vietnamese — warm red gradient */
  .pill.vi {
    background: linear-gradient(
      135deg,
      rgba(220, 38, 38, 0.72) 0%,
      rgba(185, 28, 28, 0.60) 100%
    );
  }

  /* English — cool blue gradient */
  .pill.en {
    background: linear-gradient(
      135deg,
      rgba(37, 99, 235, 0.72) 0%,
      rgba(29, 78, 216, 0.60) 100%
    );
  }

  .flag-dot {
    font-size: 18px;
    line-height: 1;
    filter: drop-shadow(0 1px 2px rgba(0,0,0,0.4));
  }

  .flag-vi,
  .flag-en {
    display: block;
  }

  .mode-label {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: #ffffff;
    text-shadow:
      0 1px 3px rgba(0, 0, 0, 0.5),
      0 0 12px rgba(255, 255, 255, 0.25);
    font-variant-numeric: tabular-nums;
  }
</style>
