import { useEffect } from "react";
import { useApp } from "../context/AppContext";
import type { WriteProgress } from "../types";
import "./ProgressBar.css";

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

export default function ProgressBar() {
  const { state, dispatch } = useApp();

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function setupListener() {
      const { listen } = await import("@tauri-apps/api/event");

      unlisten = await listen<WriteProgress>("write-progress", (event) => {
        dispatch({ type: "SET_WRITE_PROGRESS", payload: event.payload });
        if (event.payload.phase === "writing") {
          dispatch({ type: "SET_WRITE_PHASE", payload: "writing" });
        } else if (event.payload.phase === "verifying") {
          dispatch({ type: "SET_WRITE_PHASE", payload: "verifying" });
        }
      });
    }

    setupListener();
    return () => unlisten?.();
  }, [dispatch]);

  const progress = state.writeProgress;
  const percent =
    progress && progress.total_bytes > 0
      ? Math.round((progress.bytes_written / progress.total_bytes) * 100)
      : 0;

  const getStatusMessage = () => {
    switch (state.writePhase) {
      case "preparing":
        return "Preparing to write...";
      case "writing":
        return "Writing to USB...";
      case "verifying":
        return "Verifying written data...";
      case "complete":
        return "Complete! You can safely remove the USB drive.";
      case "error":
        return state.writeError || "An error occurred.";
      default:
        return "";
    }
  };

  const isError = state.writePhase === "error";
  const isComplete = state.writePhase === "complete";

  return (
    <div className={`progress-container ${isError ? "error" : ""} ${isComplete ? "complete" : ""}`}>
      <div className="progress-status">{getStatusMessage()}</div>

      {progress && !isComplete && !isError && (
        <>
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${percent}%` }} />
          </div>

          <div className="progress-stats">
            <span className="progress-percent">{percent}%</span>
            <span className="progress-bytes">
              {formatBytes(progress.bytes_written)} / {formatBytes(progress.total_bytes)}
            </span>
            <span className="progress-speed">{formatSpeed(progress.speed_bps)}</span>
            <span className="progress-eta">ETA: {formatTime(progress.eta_seconds)}</span>
          </div>
        </>
      )}

      {(isComplete || isError) && (
        <button
          className="neutral"
          onClick={() => dispatch({ type: "RESET_WRITE" })}
        >
          {isComplete ? "Write Another" : "Try Again"}
        </button>
      )}
    </div>
  );
}
