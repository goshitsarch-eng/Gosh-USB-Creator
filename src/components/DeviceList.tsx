import { useCallback, useEffect } from "react";
import { useApp } from "../context/AppContext";
import type { BlockDevice } from "../types";
import "./DeviceList.css";

const REFRESH_INTERVAL_MS = 8000;

export default function DeviceList() {
  const { state, dispatch } = useApp();
  const isWriting =
    state.writePhase === "preparing" ||
    state.writePhase === "writing" ||
    state.writePhase === "verifying";

  const refreshDevices = useCallback(async () => {
    dispatch({ type: "SET_DEVICES_LOADING", payload: true });

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const devices: BlockDevice[] = await invoke("list_devices");
      dispatch({ type: "SET_DEVICES", payload: devices });

      // Clear selection if device was unplugged
      if (
        state.selectedDevice &&
        !devices.find((d) => d.path === state.selectedDevice?.path)
      ) {
        dispatch({ type: "SET_DEVICE", payload: null });
      }
    } catch (error) {
      console.error("Failed to list devices:", error);
      dispatch({ type: "SET_DEVICES", payload: [] });
    } finally {
      dispatch({ type: "SET_DEVICES_LOADING", payload: false });
    }
  }, [dispatch, state.selectedDevice]);

  useEffect(() => {
    if (isWriting) return;
    refreshDevices();
    const interval = setInterval(refreshDevices, REFRESH_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [refreshDevices, isWriting]);

  const selectDevice = (device: BlockDevice) => {
    dispatch({ type: "SET_DEVICE", payload: device });
  };

  return (
    <div className="device-list">
      <div className="device-list-header">
        <span className="device-count">
          {state.devices.length} device{state.devices.length !== 1 ? "s" : ""} found
        </span>
        <button
          className="neutral"
          onClick={refreshDevices}
          disabled={state.devicesLoading || isWriting}
        >
          {state.devicesLoading ? "Scanning..." : "Refresh"}
        </button>
      </div>

      {state.devices.length === 0 ? (
        <div className="device-empty">
          {state.devicesLoading
            ? "Scanning for USB devices..."
            : "No removable USB devices found. Insert a USB drive and click Refresh."}
        </div>
      ) : (
        <div className="device-cards">
          {state.devices.map((device) => (
            <button
              key={device.path}
              className={`device-card ${
                state.selectedDevice?.path === device.path ? "selected" : ""
              }`}
              onClick={() => selectDevice(device)}
              disabled={isWriting}
            >
              <div className="device-main">
                <span className="device-name">{device.name}</span>
                <span className="device-size">{device.size_human}</span>
              </div>
              <div className="device-path mono">{device.path}</div>
              {device.mount_points.length > 0 && (
                <div className="device-mounts">
                  Mounted: {device.mount_points.join(", ")}
                </div>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
