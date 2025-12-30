import { useCallback } from "react";
import { useApp } from "../context/AppContext";
import type { FileInfo } from "../types";
import "./ISOSelector.css";

export default function ISOSelector() {
  const { state, dispatch } = useApp();

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
    return (
      <div className="iso-selected">
        <div className="iso-info">
          <span className="iso-name mono">{state.selectedFile.name}</span>
          <span className="iso-size">{state.selectedFile.size_human}</span>
        </div>
        <button className="neutral" onClick={clearFile}>
          Clear
        </button>
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
