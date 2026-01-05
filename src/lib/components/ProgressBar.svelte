<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { appState } from "../stores/app.svelte";
  import type { WriteProgress } from "../types";

  let unlisten: (() => void) | undefined;

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  function formatSpeed(bps: number): string {
    return formatBytes(bps) + "/s";
  }

  function formatTime(seconds: number): string {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    const mins = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${mins}m ${secs}s`;
  }

  let percent = $derived(
    appState.writeProgress && appState.writeProgress.total_bytes > 0
      ? Math.round((appState.writeProgress.bytes_written / appState.writeProgress.total_bytes) * 100)
      : 0
  );

  let statusMessage = $derived.by(() => {
    switch (appState.writePhase) {
      case "preparing":
        return "Preparing to write...";
      case "writing":
        return "Writing to USB...";
      case "verifying":
        return "Verifying written data...";
      case "complete":
        return "Complete! You can safely remove the USB drive.";
      case "error":
        return appState.writeError || "An error occurred.";
      default:
        return "";
    }
  });

  let isError = $derived(appState.writePhase === "error");
  let isComplete = $derived(appState.writePhase === "complete");

  onMount(async () => {
    unlisten = await listen<WriteProgress>("write-progress", (event) => {
      appState.setWriteProgress(event.payload);
      if (event.payload.phase === "writing") {
        appState.setWritePhase("writing");
      } else if (event.payload.phase === "verifying") {
        appState.setWritePhase("verifying");
      }
    });
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div class="progress-container" class:error={isError} class:complete={isComplete}>
  <div class="progress-status">{statusMessage}</div>

  {#if appState.writeProgress && !isComplete && !isError}
    <div class="progress-bar">
      <div class="progress-fill" style="width: {percent}%"></div>
    </div>

    <div class="progress-stats">
      <span class="progress-percent">{percent}%</span>
      <span class="progress-bytes">
        {formatBytes(appState.writeProgress.bytes_written)} / {formatBytes(appState.writeProgress.total_bytes)}
      </span>
      <span class="progress-speed">{formatSpeed(appState.writeProgress.speed_bps)}</span>
      <span class="progress-eta">ETA: {formatTime(appState.writeProgress.eta_seconds)}</span>
    </div>
  {/if}

  {#if isComplete || isError}
    <button
      class="neutral"
      onclick={() => appState.resetWrite()}
    >
      {isComplete ? "Write Another" : "Try Again"}
    </button>
  {/if}
</div>

<style>
  .progress-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .progress-status {
    font-size: 0.875rem;
    font-weight: 500;
  }

  .progress-container.error .progress-status {
    color: var(--color-danger);
  }

  .progress-container.complete .progress-status {
    color: var(--color-success);
  }

  .progress-bar {
    height: 8px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.3s ease;
  }

  .progress-stats {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .progress-percent {
    font-weight: 500;
    color: var(--text-primary);
  }

  button.neutral {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 0.375rem 0.75rem;
    border-radius: 4px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background 0.15s;
    align-self: flex-start;
  }

  button.neutral:hover {
    background: var(--bg-hover);
  }
</style>
