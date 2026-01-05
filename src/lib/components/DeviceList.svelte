<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../stores/app.svelte";
  import type { BlockDevice } from "../types";

  const REFRESH_INTERVAL_MS = 8000;
  let interval: ReturnType<typeof setInterval> | undefined;

  async function refreshDevices() {
    appState.setDevicesLoading(true);

    try {
      const devices: BlockDevice[] = await invoke("list_devices");
      appState.setDevices(devices);

      // Clear selection if device was unplugged
      if (
        appState.selectedDevice &&
        !devices.find((d) => d.path === appState.selectedDevice?.path)
      ) {
        appState.setDevice(null);
      }
    } catch (error) {
      console.error("Failed to list devices:", error);
      appState.setDevices([]);
    } finally {
      appState.setDevicesLoading(false);
    }
  }

  function selectDevice(device: BlockDevice) {
    appState.setDevice(device);
  }

  onMount(() => {
    refreshDevices();
    interval = setInterval(() => {
      if (!appState.isWriting) {
        refreshDevices();
      }
    }, REFRESH_INTERVAL_MS);
  });

  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
</script>

<div class="device-list">
  <div class="device-list-header">
    <span class="device-count">
      {appState.devices.length} device{appState.devices.length !== 1 ? "s" : ""} found
    </span>
    <button
      class="neutral"
      onclick={refreshDevices}
      disabled={appState.devicesLoading || appState.isWriting}
    >
      {appState.devicesLoading ? "Scanning..." : "Refresh"}
    </button>
  </div>

  {#if appState.devices.length === 0}
    <div class="device-empty">
      {appState.devicesLoading
        ? "Scanning for USB devices..."
        : "No removable USB devices found. Insert a USB drive and click Refresh."}
    </div>
  {:else}
    <div class="device-cards">
      {#each appState.devices as device (device.path)}
        <button
          class="device-card"
          class:selected={appState.selectedDevice?.path === device.path}
          onclick={() => selectDevice(device)}
          disabled={appState.isWriting}
        >
          <div class="device-main">
            <span class="device-name">{device.name}</span>
            <span class="device-size">{device.size_human}</span>
          </div>
          <div class="device-path mono">{device.path}</div>
          {#if device.mount_points.length > 0}
            <div class="device-mounts">
              Mounted: {device.mount_points.join(", ")}
            </div>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .device-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .device-list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .device-count {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .device-empty {
    padding: 2rem;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .device-cards {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .device-card {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
    background: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s;
  }

  .device-card:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .device-card.selected {
    border-color: var(--color-primary);
    background: var(--bg-active);
  }

  .device-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .device-main {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .device-name {
    font-weight: 500;
  }

  .device-size {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .device-path {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .device-mounts {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .mono {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
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
  }

  button.neutral:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  button.neutral:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
