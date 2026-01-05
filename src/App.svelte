<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { sendNotification, isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
  import { appState } from "./lib/stores/app.svelte";
  import ISOSelector from "./lib/components/ISOSelector.svelte";
  import VerificationPanel from "./lib/components/VerificationPanel.svelte";
  import DeviceList from "./lib/components/DeviceList.svelte";
  import ProgressBar from "./lib/components/ProgressBar.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import ModeToggle from "./lib/components/ModeToggle.svelte";

  type NavItem = "write" | "settings";

  let activeNav = $state<NavItem>("write");

  async function handleWrite() {
    if (!appState.selectedFile || !appState.selectedDevice) return;

    const confirmed = await ask(
      `This will erase all data on ${appState.selectedDevice.name} (${appState.selectedDevice.size_human}). Continue?`,
      {
        title: "Confirm Write",
        kind: "warning",
        okLabel: "Write",
        cancelLabel: "Cancel",
      }
    );
    if (!confirmed) return;

    appState.setWritePhase("preparing");
    appState.setWriteError(null);

    const devicePath = appState.selectedDevice.path;

    try {
      await invoke("write_iso_to_device", {
        isoPath: appState.selectedFile.path,
        devicePath: devicePath,
        verify: appState.verifyAfterWrite,
      });
      appState.setWritePhase("complete");

      // Post-write actions (Advanced mode only)
      if (appState.mode === "advanced") {
        // Show notification
        if (appState.showNotification) {
          try {
            let hasPermission = await isPermissionGranted();
            if (!hasPermission) {
              const permission = await requestPermission();
              hasPermission = permission === "granted";
            }
            if (hasPermission) {
              sendNotification({
                title: "Write Complete",
                body: `Successfully wrote ${appState.selectedFile!.name} to USB drive.`,
              });
            }
          } catch (e) {
            console.warn("Notification not available:", e);
          }
        }

        // Auto-eject
        if (appState.autoEject) {
          try {
            await invoke("eject_device", { devicePath });
          } catch (e) {
            console.warn("Failed to eject:", e);
          }
        }
      }
    } catch (error) {
      appState.setWriteError(String(error));
    }
  }
</script>

<div class="app">
  <nav class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Gosh USB Creator</span>
    </div>
    <ul class="sidebar-nav">
      <li>
        <button
          class="sidebar-item"
          class:active={activeNav === "write"}
          onclick={() => activeNav = "write"}
        >
          Write Image
        </button>
      </li>
      <li>
        <button
          class="sidebar-item"
          class:active={activeNav === "settings"}
          onclick={() => activeNav = "settings"}
        >
          Settings
        </button>
      </li>
    </ul>
    <ModeToggle />
  </nav>

  <main class="main">
    {#if activeNav === "write"}
      <div class="card">
        <div class="card-header">Source Image</div>
        <div class="card-body">
          <ISOSelector />
        </div>
      </div>

      {#if appState.selectedFile}
        <div class="card">
          <div class="card-header">Verify Checksum</div>
          <div class="card-body">
            <VerificationPanel />
          </div>
        </div>
      {/if}

      <div class="card">
        <div class="card-header">Target Device</div>
        <div class="card-body">
          <DeviceList />
        </div>
      </div>

      {#if appState.isWriting || appState.writePhase === "complete" || appState.writePhase === "error"}
        <div class="card">
          <div class="card-header">Progress</div>
          <div class="card-body">
            <ProgressBar />
          </div>
        </div>
      {/if}

      <div class="card">
        <div class="card-body action-bar">
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={appState.verifyAfterWrite}
              onchange={(e) => appState.setVerifyAfterWrite(e.currentTarget.checked)}
              disabled={appState.isWriting}
            />
            Verify after writing
          </label>

          <button
            class={appState.canWrite ? "safe" : "danger"}
            onclick={handleWrite}
            disabled={!appState.canWrite || appState.isWriting}
          >
            {appState.isWriting ? "Writing..." : "Write"}
          </button>
        </div>
      </div>
    {:else}
      <Settings />
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    min-height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .sidebar {
    width: 200px;
    min-width: 200px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 1rem 0;
  }

  .sidebar-header {
    padding: 0 1rem 1rem;
    border-bottom: 1px solid var(--border);
    margin-bottom: 1rem;
  }

  .sidebar-title {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .sidebar-nav {
    list-style: none;
    padding: 0;
    margin: 0;
    flex: 1;
  }

  .sidebar-item {
    display: block;
    width: 100%;
    padding: 0.625rem 1rem;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.875rem;
    transition: background 0.15s;
  }

  .sidebar-item:hover {
    background: var(--bg-hover);
  }

  .sidebar-item.active {
    background: var(--bg-active);
    font-weight: 500;
  }

  .main {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .card-header {
    padding: 0.75rem 1rem;
    font-weight: 500;
    font-size: 0.875rem;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
  }

  .card-body {
    padding: 1rem;
  }

  .action-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .checkbox-label input {
    cursor: pointer;
  }

  button.safe {
    background: var(--color-success);
    color: white;
    border: none;
    padding: 0.5rem 1.5rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  button.safe:hover:not(:disabled) {
    opacity: 0.9;
  }

  button.safe:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.danger {
    background: var(--color-danger);
    color: white;
    border: none;
    padding: 0.5rem 1.5rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    opacity: 0.5;
  }
</style>
