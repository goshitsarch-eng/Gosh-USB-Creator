<script lang="ts">
  import { appState } from "../stores/app.svelte";
  import type { AppMode } from "../types";

  const modes: AppMode[] = ["standard", "advanced"];
</script>

<div class="mode-toggle">
  {#each modes as mode}
    <button
      class="mode-option"
      class:selected={appState.mode === mode}
      onclick={() => appState.setMode(mode)}
      disabled={appState.isWriting}
    >
      {mode.charAt(0).toUpperCase() + mode.slice(1)}
    </button>
  {/each}
</div>

<style>
  .mode-toggle {
    display: flex;
    margin: 1rem;
    gap: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }

  .mode-option {
    flex: 1;
    padding: 0.5rem;
    background: var(--bg-tertiary);
    border: none;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s;
  }

  .mode-option:first-child {
    border-right: 1px solid var(--border);
  }

  .mode-option:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .mode-option.selected {
    background: var(--bg-active);
    color: var(--text-primary);
    font-weight: 500;
  }

  .mode-option:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
