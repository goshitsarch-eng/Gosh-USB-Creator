import { useCallback, useEffect, useState } from "react";
import { useApp } from "../context/AppContext";
import type { FileInfo, ImageValidation } from "../types";
import "./ISOSelector.css";

const FORMAT_INFO: Record<string, string> = {
  "ISO 9660": "Standard format for optical disc images (CDs, DVDs). Used by most Linux distributions and Windows installation media.",
  "Disk Image (MBR)": "Master Boot Record — Legacy boot format supporting up to 4 primary partitions and 2TB drives. Compatible with older BIOS systems.",
  "Disk Image (GPT)": "GUID Partition Table — Modern boot format supporting unlimited partitions and drives larger than 2TB. Required for UEFI boot.",
  "Unknown": "Format not recognized. The file may still work but could not be validated as a standard disk image.",
};

export default function ISOSelector() {
  const { state, dispatch } = useApp();
  const [showFormatInfo, setShowFormatInfo] = useState(false);

  // Auto-validate image in advanced mode
  useEffect(() => {
    async function validateImage() {
      if (state.mode !== "advanced" || !state.selectedFile) return;
      if (state.imageValidation !== null || state.imageValidationLoading) return;

      dispatch({ type: "SET_IMAGE_VALIDATION_LOADING", payload: true });
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const deviceSize = state.selectedDevice?.size ?? null;
        const validation: ImageValidation = await invoke("validate_image", {
          path: state.selectedFile.path,
          deviceSize,
        });
        dispatch({ type: "SET_IMAGE_VALIDATION", payload: validation });
      } catch (error) {
        console.error("Failed to validate image:", error);
      } finally {
        dispatch({ type: "SET_IMAGE_VALIDATION_LOADING", payload: false });
      }
    }
    validateImage();
  }, [state.mode, state.selectedFile, state.selectedDevice, state.imageValidation, state.imageValidationLoading, dispatch]);

  const selectFile = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { invoke } = await import("@tauri-apps/api/core");

      const selected = await open({
        multiple: false,
        filters: [
          { name: "ISO Images", extensions: ["iso", "img"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });

      if (selected && typeof selected === "string") {
        const fileInfo: FileInfo = await invoke("get_file_info", { path: selected });
        dispatch({ type: "SET_FILE", payload: fileInfo });
      }
    } catch (error) {
      console.error("Failed to select file:", error);
    }
  }, [dispatch]);

  const clearFile = useCallback(() => {
    dispatch({ type: "SET_FILE", payload: null });
  }, [dispatch]);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      const files = e.dataTransfer.files;
      if (files.length > 0) {
        const path = (files[0] as File & { path?: string }).path;
        if (path) {
          try {
            const { invoke } = await import("@tauri-apps/api/core");
            const fileInfo: FileInfo = await invoke("get_file_info", { path });
            dispatch({ type: "SET_FILE", payload: fileInfo });
          } catch (error) {
            console.error("Failed to get file info:", error);
          }
        }
      }
    },
    [dispatch]
  );

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  if (state.selectedFile) {
    const validation = state.imageValidation;
    const isAdvanced = state.mode === "advanced";

    return (
      <div className="iso-selected">
        <div className="iso-info">
          <span className="iso-name mono">{state.selectedFile.name}</span>
          <span className="iso-size">{state.selectedFile.size_human}</span>
        </div>
        <button className="neutral" onClick={clearFile}>
          Clear
        </button>

        {isAdvanced && (
          <div className="iso-validation">
            {state.imageValidationLoading && (
              <span className="validation-loading">Validating...</span>
            )}
            {validation && (
              <>
                <div className={`validation-status ${validation.is_valid ? "valid" : "invalid"}`}>
                  <button
                    type="button"
                    className="validation-format-btn"
                    onClick={() => setShowFormatInfo(!showFormatInfo)}
                    title="Click for more info"
                  >
                    {validation.format}
                    <span className="info-icon">?</span>
                  </button>
                  <span className="validation-badge">
                    {validation.is_valid ? "Valid" : "Invalid"}
                  </span>
                </div>
                {showFormatInfo && (
                  <div className="format-info-popup">
                    <p>{FORMAT_INFO[validation.format] || FORMAT_INFO["Unknown"]}</p>
                  </div>
                )}
                {validation.errors.length > 0 && (
                  <ul className="validation-errors">
                    {validation.errors.map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                  </ul>
                )}
                {validation.warnings.length > 0 && (
                  <ul className="validation-warnings">
                    {validation.warnings.map((warn, i) => (
                      <li key={i}>{warn}</li>
                    ))}
                  </ul>
                )}
              </>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <div
      className="iso-dropzone"
      onClick={selectFile}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === "Enter" && selectFile()}
    >
      <div className="iso-dropzone-content">
        <span className="iso-dropzone-icon">+</span>
        <span>Click to select or drag ISO file here</span>
      </div>
    </div>
  );
}
