import { useApp } from "../context/AppContext";
import type { AppMode } from "../types";
import "./ModeToggle.css";

export default function ModeToggle() {
  const { state, dispatch } = useApp();

  const isWriting =
    state.writePhase === "preparing" ||
    state.writePhase === "writing" ||
    state.writePhase === "verifying";

  return (
    <div className="mode-toggle">
      {(["standard", "advanced"] as AppMode[]).map((mode) => (
        <button
          key={mode}
          className={`mode-option ${state.mode === mode ? "selected" : ""}`}
          onClick={() => dispatch({ type: "SET_MODE", payload: mode })}
          disabled={isWriting}
        >
          {mode.charAt(0).toUpperCase() + mode.slice(1)}
        </button>
      ))}
    </div>
  );
}
