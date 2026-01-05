<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../stores/app.svelte";

  let checksumMatch = $derived(
    appState.calculatedChecksum &&
    appState.expectedChecksum.trim().length > 0 &&
    appState.calculatedChecksum.toLowerCase() === appState.expectedChecksum.trim().toLowerCase()
  );

  let checksumMismatch = $derived(
    appState.calculatedChecksum &&
    appState.expectedChecksum.trim().length > 0 &&
    appState.calculatedChecksum.toLowerCase() !== appState.expectedChecksum.trim().toLowerCase()
  );

  async function calculateChecksum() {
    if (!appState.selectedFile) return;

    appState.setChecksumLoading(true);
    appState.setChecksum(null);

    try {
      const checksum: string = await invoke("calculate_checksum", {
        path: appState.selectedFile.path,
        algorithm: appState.checksumAlgorithm,
      });
      appState.setChecksum(checksum);
    } catch (error) {
      console.error("Failed to calculate checksum:", error);
    } finally {
      appState.setChecksumLoading(false);
    }
  }
</script>

<div class="verification-panel">
  <div class="verification-row">
    <select
      value={appState.checksumAlgorithm}
      onchange={(e) => appState.setChecksumAlgorithm(e.currentTarget.value as "sha256" | "md5")}
      disabled={appState.checksumLoading}
    >
      <option value="sha256">SHA-256</option>
      <option value="md5">MD5</option>
    </select>
    <button
      class="neutral"
      onclick={calculateChecksum}
      disabled={appState.checksumLoading || !appState.selectedFile}
    >
      {appState.checksumLoading ? "Calculating..." : "Calculate"}
    </button>
  </div>

  {#if appState.calculatedChecksum}
    <div class="checksum-result">
      <label>Calculated:</label>
      <code class="checksum-value">{appState.calculatedChecksum}</code>
    </div>
  {/if}

  <div class="checksum-compare">
    <label for="expected-checksum">Expected (paste to compare):</label>
    <input
      id="expected-checksum"
      type="text"
      class="mono"
      placeholder="Paste expected checksum here..."
      value={appState.expectedChecksum}
      oninput={(e) => appState.setExpectedChecksum(e.currentTarget.value)}
    />
    {#if checksumMatch}
      <span class="checksum-status match">Checksums match</span>
    {/if}
    {#if checksumMismatch}
      <span class="checksum-status mismatch">Checksums do NOT match</span>
    {/if}
  </div>
</div>

<style>
  .verification-panel {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .verification-row {
    display: flex;
    gap: 0.5rem;
  }

  select {
    padding: 0.375rem 0.75rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.875rem;
    cursor: pointer;
  }

  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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

  .checksum-result {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .checksum-result label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .checksum-value {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.75rem;
    padding: 0.5rem;
    background: var(--bg-tertiary);
    border-radius: 4px;
    word-break: break-all;
  }

  .checksum-compare {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .checksum-compare label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .checksum-compare input {
    padding: 0.5rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.75rem;
  }

  .checksum-compare input::placeholder {
    color: var(--text-secondary);
  }

  .checksum-status {
    font-size: 0.875rem;
    font-weight: 500;
    padding: 0.25rem 0;
  }

  .checksum-status.match {
    color: var(--color-success);
  }

  .checksum-status.mismatch {
    color: var(--color-danger);
  }

  .mono {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
  }
</style>
