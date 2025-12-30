import { useCallback } from "react";
import { useApp } from "../context/AppContext";
import "./VerificationPanel.css";

export default function VerificationPanel() {
  const { state, dispatch } = useApp();

  const calculateChecksum = useCallback(async () => {
    if (!state.selectedFile) return;

    dispatch({ type: "SET_CHECKSUM_LOADING", payload: true });
    dispatch({ type: "SET_CHECKSUM", payload: null });

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const checksum: string = await invoke("calculate_checksum", {
        path: state.selectedFile.path,
        algorithm: state.checksumAlgorithm,
      });
      dispatch({ type: "SET_CHECKSUM", payload: checksum });
    } catch (error) {
      console.error("Failed to calculate checksum:", error);
    } finally {
      dispatch({ type: "SET_CHECKSUM_LOADING", payload: false });
    }
  }, [state.selectedFile, state.checksumAlgorithm, dispatch]);

  const checksumMatch =
    state.calculatedChecksum &&
    state.expectedChecksum.trim().length > 0 &&
    state.calculatedChecksum.toLowerCase() === state.expectedChecksum.trim().toLowerCase();

  const checksumMismatch =
    state.calculatedChecksum &&
    state.expectedChecksum.trim().length > 0 &&
    state.calculatedChecksum.toLowerCase() !== state.expectedChecksum.trim().toLowerCase();

  return (
    <div className="verification-panel">
      <div className="verification-row">
        <select
          value={state.checksumAlgorithm}
          onChange={(e) =>
            dispatch({
              type: "SET_CHECKSUM_ALGORITHM",
              payload: e.target.value as "sha256" | "md5",
            })
          }
          disabled={state.checksumLoading}
        >
          <option value="sha256">SHA-256</option>
          <option value="md5">MD5</option>
        </select>
        <button
          className="neutral"
          onClick={calculateChecksum}
          disabled={state.checksumLoading || !state.selectedFile}
        >
          {state.checksumLoading ? "Calculating..." : "Calculate"}
        </button>
      </div>

      {state.calculatedChecksum && (
        <div className="checksum-result">
          <label>Calculated:</label>
          <code className="checksum-value">{state.calculatedChecksum}</code>
        </div>
      )}

      <div className="checksum-compare">
        <label htmlFor="expected-checksum">Expected (paste to compare):</label>
        <input
          id="expected-checksum"
          type="text"
          className="mono"
          placeholder="Paste expected checksum here..."
          value={state.expectedChecksum}
          onChange={(e) =>
            dispatch({ type: "SET_EXPECTED_CHECKSUM", payload: e.target.value })
          }
        />
        {checksumMatch && (
          <span className="checksum-status match">Checksums match</span>
        )}
        {checksumMismatch && (
          <span className="checksum-status mismatch">Checksums do NOT match</span>
        )}
      </div>
    </div>
  );
}
