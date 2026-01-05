<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { appState } from "../stores/app.svelte";
  import type { FileInfo, ImageValidation } from "../types";

  const FORMAT_INFO: Record<string, string> = {
    "ISO 9660": "Standard format for optical disc images (CDs, DVDs). Used by most Linux distributions and Windows installation media.",
    "Disk Image (MBR)": "Master Boot Record - Legacy boot format supporting up to 4 primary partitions and 2TB drives. Compatible with older BIOS systems.",
    "Disk Image (GPT)": "GUID Partition Table - Modern boot format supporting unlimited partitions and drives larger than 2TB. Required for UEFI boot.",
    "Unknown": "Format not recognized. The file may still work but could not be validated as a standard disk image.",
  };

  let showFormatInfo = $state(false);

  // Auto-validate image in advanced mode
  $effect(() => {
    if (appState.mode !== "advanced" || !appState.selectedFile) return;
    if (appState.imageValidation !== null || appState.imageValidationLoading) return;

    validateImage();
  });

  async function validateImage() {
    if (!appState.selectedFile) return;

    appState.setImageValidationLoading(true);
    try {
      const deviceSize = appState.selectedDevice?.size ?? null;
      const validation: ImageValidation = await invoke("validate_image", {
        path: appState.selectedFile.path,
        deviceSize,
      });
      appState.setImageValidation(validation);
    } catch (error) {
      console.error("Failed to validate image:", error);
    } finally {
      appState.setImageValidationLoading(false);
    }
  }

  async function selectFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "ISO Images", extensions: ["iso", "img"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });

      if (selected && typeof selected === "string") {
        const fileInfo: FileInfo = await invoke("get_file_info", { path: selected });
        appState.setFile(fileInfo);
      }
    } catch (error) {
      console.error("Failed to select file:", error);
    }
  }

  function clearFile() {
    appState.setFile(null);
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0] as File & { path?: string };
      const path = file.path;
      if (path) {
        handleFilePath(path);
      }
    }
  }

  async function handleFilePath(path: string) {
    try {
      const fileInfo: FileInfo = await invoke("get_file_info", { path });
      appState.setFile(fileInfo);
    } catch (error) {
      console.error("Failed to get file info:", error);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      selectFile();
    }
  }
</script>

{#if appState.selectedFile}
  <div class="iso-selected">
    <div class="iso-info">
      <span class="iso-name mono">{appState.selectedFile.name}</span>
      <span class="iso-size">{appState.selectedFile.size_human}</span>
    </div>
    <button class="neutral" onclick={clearFile}>
      Clear
    </button>

    {#if appState.mode === "advanced"}
      <div class="iso-validation">
        {#if appState.imageValidationLoading}
          <span class="validation-loading">Validating...</span>
        {/if}
        {#if appState.imageValidation}
          <div class="validation-status" class:valid={appState.imageValidation.is_valid} class:invalid={!appState.imageValidation.is_valid}>
            <button
              type="button"
              class="validation-format-btn"
              onclick={() => showFormatInfo = !showFormatInfo}
              title="Click for more info"
            >
              {appState.imageValidation.format}
              <span class="info-icon">?</span>
            </button>
            <span class="validation-badge">
              {appState.imageValidation.is_valid ? "Valid" : "Invalid"}
            </span>
          </div>
          {#if showFormatInfo}
            <div class="format-info-popup">
              <p>{FORMAT_INFO[appState.imageValidation.format] || FORMAT_INFO["Unknown"]}</p>
            </div>
          {/if}
          {#if appState.imageValidation.errors.length > 0}
            <ul class="validation-errors">
              {#each appState.imageValidation.errors as err}
                <li>{err}</li>
              {/each}
            </ul>
          {/if}
          {#if appState.imageValidation.warnings.length > 0}
            <ul class="validation-warnings">
              {#each appState.imageValidation.warnings as warn}
                <li>{warn}</li>
              {/each}
            </ul>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
{:else}
  <div
    class="iso-dropzone"
    onclick={selectFile}
    ondrop={handleDrop}
    ondragover={handleDragOver}
    role="button"
    tabindex="0"
    onkeydown={handleKeyDown}
  >
    <div class="iso-dropzone-content">
      <span class="iso-dropzone-icon">+</span>
      <span>Click to select or drag ISO file here</span>
    </div>
  </div>
{/if}

<style>
  .iso-selected {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .iso-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: var(--bg-tertiary);
    border-radius: 6px;
  }

  .iso-name {
    font-weight: 500;
    word-break: break-all;
  }

  .iso-size {
    color: var(--text-secondary);
    font-size: 0.875rem;
    white-space: nowrap;
    margin-left: 1rem;
  }

  .iso-dropzone {
    border: 2px dashed var(--border);
    border-radius: 8px;
    padding: 3rem 2rem;
    text-align: center;
    cursor: pointer;
    transition: all 0.15s;
  }

  .iso-dropzone:hover {
    border-color: var(--color-primary);
    background: var(--bg-hover);
  }

  .iso-dropzone-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-secondary);
  }

  .iso-dropzone-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .iso-validation {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    background: var(--bg-tertiary);
    border-radius: 6px;
  }

  .validation-status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .validation-status.valid .validation-badge {
    background: var(--color-success);
  }

  .validation-status.invalid .validation-badge {
    background: var(--color-danger);
  }

  .validation-format-btn {
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
  }

  .info-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    background: var(--bg-hover);
    font-size: 0.625rem;
  }

  .validation-badge {
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    color: white;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .validation-loading {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .format-info-popup {
    padding: 0.75rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .format-info-popup p {
    margin: 0;
  }

  .validation-errors {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .validation-errors li {
    color: var(--color-danger);
    font-size: 0.875rem;
  }

  .validation-warnings {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .validation-warnings li {
    color: var(--color-warning);
    font-size: 0.875rem;
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
    align-self: flex-start;
  }

  button.neutral:hover {
    background: var(--bg-hover);
  }
</style>
