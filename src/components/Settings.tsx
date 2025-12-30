import { useApp } from "../context/AppContext";
import type { Theme } from "../types";
import "./Settings.css";

export default function Settings() {
  const { state, dispatch } = useApp();

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
