<script lang="ts">
  import { appState } from "../stores/app.svelte";
  import type { Theme } from "../types";

  const themes: Theme[] = ["light", "dark", "system"];
</script>

<div class="card">
  <div class="card-header">Theme</div>
  <div class="card-body">
    <div class="settings-options">
      {#each themes as theme}
        <button
          class="settings-option"
          class:selected={appState.theme === theme}
          onclick={() => appState.setTheme(theme)}
        >
          {theme.charAt(0).toUpperCase() + theme.slice(1)}
        </button>
      {/each}
    </div>
  </div>
</div>

<div class="card">
  <div class="card-header">Defaults</div>
  <div class="card-body">
    <label class="settings-checkbox">
      <input
        type="checkbox"
        checked={appState.verifyAfterWrite}
        onchange={(e) => appState.setVerifyAfterWrite(e.currentTarget.checked)}
      />
      Verify after writing
    </label>
    <p class="settings-hint">
      Reads back written data to ensure it matches the source file.
    </p>
  </div>
</div>

{#if appState.mode === "advanced"}
  <div class="card">
    <div class="card-header">Post-Write Actions</div>
    <div class="card-body settings-actions">
      <label class="settings-checkbox">
        <input
          type="checkbox"
          checked={appState.autoEject}
          onchange={(e) => appState.setAutoEject(e.currentTarget.checked)}
        />
        Auto-eject device after write
      </label>
      <label class="settings-checkbox">
        <input
          type="checkbox"
          checked={appState.showNotification}
          onchange={(e) => appState.setShowNotification(e.currentTarget.checked)}
        />
        Show system notification on completion
      </label>
    </div>
  </div>
{/if}

<div class="card">
  <div class="card-header">About</div>
  <div class="card-body settings-about">
    <p><strong>Gosh USB Creator</strong></p>
    <p class="settings-meta">Version 1.1.1</p>
    <p class="settings-meta">AGPL-3.0</p>
  </div>
</div>

<style>
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

  .settings-options {
    display: flex;
    gap: 0.5rem;
  }

  .settings-option {
    padding: 0.5rem 1rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.15s;
  }

  .settings-option:hover {
    background: var(--bg-hover);
  }

  .settings-option.selected {
    background: var(--bg-active);
    border-color: var(--color-primary);
  }

  .settings-checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .settings-checkbox input {
    cursor: pointer;
  }

  .settings-hint {
    margin: 0.5rem 0 0;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .settings-actions {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .settings-about p {
    margin: 0;
  }

  .settings-about p + p {
    margin-top: 0.25rem;
  }

  .settings-meta {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }
</style>
