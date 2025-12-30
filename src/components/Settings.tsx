import { useApp } from "../context/AppContext";
import type { Theme } from "../types";
import "./Settings.css";

export default function Settings() {
  const { state, dispatch } = useApp();
  const isAdvanced = state.mode === "advanced";

  return (
    <>
      <div className="card">
        <div className="card-header">Theme</div>
        <div className="card-body">
          <div className="settings-options">
            {(["light", "dark", "system"] as Theme[]).map((theme) => (
              <button
                key={theme}
                className={`settings-option ${state.theme === theme ? "selected" : ""}`}
                onClick={() => dispatch({ type: "SET_THEME", payload: theme })}
              >
                {theme.charAt(0).toUpperCase() + theme.slice(1)}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-header">Defaults</div>
        <div className="card-body">
          <label className="settings-checkbox">
            <input
              type="checkbox"
              checked={state.verifyAfterWrite}
              onChange={(e) =>
                dispatch({ type: "SET_VERIFY_AFTER_WRITE", payload: e.target.checked })
              }
            />
            Verify after writing
          </label>
          <p className="settings-hint">
            Reads back written data to ensure it matches the source file.
          </p>
        </div>
      </div>

      {isAdvanced && (
        <div className="card">
          <div className="card-header">Post-Write Actions</div>
          <div className="card-body settings-actions">
            <label className="settings-checkbox">
              <input
                type="checkbox"
                checked={state.autoEject}
                onChange={(e) =>
                  dispatch({ type: "SET_AUTO_EJECT", payload: e.target.checked })
                }
              />
              Auto-eject device after write
            </label>
            <label className="settings-checkbox">
              <input
                type="checkbox"
                checked={state.showNotification}
                onChange={(e) =>
                  dispatch({ type: "SET_SHOW_NOTIFICATION", payload: e.target.checked })
                }
              />
              Show system notification on completion
            </label>
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-header">About</div>
        <div className="card-body settings-about">
          <p><strong>Gosh USB Creator</strong></p>
          <p className="settings-meta">Version 0.1.0</p>
          <p className="settings-meta">AGPL-3.0</p>
        </div>
      </div>
    </>
  );
}
