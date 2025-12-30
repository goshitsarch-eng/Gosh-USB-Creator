import { useState } from "react";
import { useApp } from "../context/AppContext";
import ISOSelector from "./ISOSelector";
import VerificationPanel from "./VerificationPanel";
import DeviceList from "./DeviceList";
import ProgressBar from "./ProgressBar";
import Settings from "./Settings";
import "./App.css";

type NavItem = "write" | "settings";

export default function App() {
  const { state, dispatch } = useApp();
  const [activeNav, setActiveNav] = useState<NavItem>("write");

  const canWrite =
    state.selectedFile !== null &&
    state.selectedDevice !== null &&
    state.writePhase === "idle";

  const isWriting =
    state.writePhase === "preparing" ||
    state.writePhase === "writing" ||
    state.writePhase === "verifying";

  async function handleWrite() {
    if (!state.selectedFile || !state.selectedDevice) return;

    const confirmed = await confirmWrite();
    if (!confirmed) return;

    dispatch({ type: "SET_WRITE_PHASE", payload: "preparing" });
    dispatch({ type: "SET_WRITE_ERROR", payload: null });

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("write_iso_to_device", {
        isoPath: state.selectedFile.path,
        devicePath: state.selectedDevice.path,
        verify: state.verifyAfterWrite,
      });
      dispatch({ type: "SET_WRITE_PHASE", payload: "complete" });
    } catch (error) {
      dispatch({ type: "SET_WRITE_ERROR", payload: String(error) });
    }
  }

  async function confirmWrite(): Promise<boolean> {
    const { ask } = await import("@tauri-apps/plugin-dialog");
    return ask(
      `This will erase all data on ${state.selectedDevice?.name} (${state.selectedDevice?.size_human}). Continue?`,
      {
        title: "Confirm Write",
        kind: "warning",
        okLabel: "Write",
        cancelLabel: "Cancel",
      }
    );
  }

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="sidebar-header">
          <span className="sidebar-title">Gosh USB Creator</span>
        </div>
        <ul className="sidebar-nav">
          <li>
            <button
              className={`sidebar-item ${activeNav === "write" ? "active" : ""}`}
              onClick={() => setActiveNav("write")}
            >
              Write Image
            </button>
          </li>
          <li>
            <button
              className={`sidebar-item ${activeNav === "settings" ? "active" : ""}`}
              onClick={() => setActiveNav("settings")}
            >
              Settings
            </button>
          </li>
        </ul>
      </nav>

      <main className="main">
        {activeNav === "write" && (
          <>
            <div className="card">
              <div className="card-header">Source Image</div>
              <div className="card-body">
                <ISOSelector />
              </div>
            </div>

            {state.selectedFile && (
              <div className="card">
                <div className="card-header">Verify Checksum</div>
                <div className="card-body">
                  <VerificationPanel />
                </div>
              </div>
            )}

            <div className="card">
              <div className="card-header">Target Device</div>
              <div className="card-body">
                <DeviceList />
              </div>
            </div>

            {(isWriting || state.writePhase === "complete" || state.writePhase === "error") && (
              <div className="card">
                <div className="card-header">Progress</div>
                <div className="card-body">
                  <ProgressBar />
                </div>
              </div>
            )}

            <div className="card">
              <div className="card-body action-bar">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={state.verifyAfterWrite}
                    onChange={(e) =>
                      dispatch({ type: "SET_VERIFY_AFTER_WRITE", payload: e.target.checked })
                    }
                    disabled={isWriting}
                  />
                  Verify after writing
                </label>

                <button
                  className={canWrite ? "safe" : "danger"}
                  onClick={handleWrite}
                  disabled={!canWrite || isWriting}
                >
                  {isWriting ? "Writing..." : "Write"}
                </button>
              </div>
            </div>
          </>
        )}

        {activeNav === "settings" && <Settings />}
      </main>
    </div>
  );
}
