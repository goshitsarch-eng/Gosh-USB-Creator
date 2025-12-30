import { createContext, useContext, useReducer, useEffect, type ReactNode, type Dispatch } from "react";
import type { AppState, AppAction, Theme, AppMode } from "../types";

const initialState: AppState = {
  selectedFile: null,
  selectedDevice: null,
  devices: [],
  devicesLoading: false,
  calculatedChecksum: null,
  checksumAlgorithm: "sha256",
  checksumLoading: false,
  expectedChecksum: "",
  writePhase: "idle",
  writeProgress: null,
  writeError: null,
  verifyAfterWrite: true,
  theme: "system",
  mode: "standard",
  imageValidation: null,
  imageValidationLoading: false,
  autoEject: false,
  showNotification: true,
};

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "SET_FILE":
      return { ...state, selectedFile: action.payload, calculatedChecksum: null, imageValidation: null };
    case "SET_DEVICE":
      return { ...state, selectedDevice: action.payload };
    case "SET_DEVICES":
      return { ...state, devices: action.payload };
    case "SET_DEVICES_LOADING":
      return { ...state, devicesLoading: action.payload };
    case "SET_CHECKSUM":
      return { ...state, calculatedChecksum: action.payload };
    case "SET_CHECKSUM_ALGORITHM":
      return { ...state, checksumAlgorithm: action.payload, calculatedChecksum: null };
    case "SET_CHECKSUM_LOADING":
      return { ...state, checksumLoading: action.payload };
    case "SET_EXPECTED_CHECKSUM":
      return { ...state, expectedChecksum: action.payload };
    case "SET_WRITE_PHASE":
      return { ...state, writePhase: action.payload };
    case "SET_WRITE_PROGRESS":
      return { ...state, writeProgress: action.payload };
    case "SET_WRITE_ERROR":
      return { ...state, writeError: action.payload, writePhase: action.payload ? "error" : state.writePhase };
    case "SET_VERIFY_AFTER_WRITE":
      return { ...state, verifyAfterWrite: action.payload };
    case "SET_THEME":
      return { ...state, theme: action.payload };
    case "SET_MODE":
      return { ...state, mode: action.payload };
    case "SET_IMAGE_VALIDATION":
      return { ...state, imageValidation: action.payload };
    case "SET_IMAGE_VALIDATION_LOADING":
      return { ...state, imageValidationLoading: action.payload };
    case "SET_AUTO_EJECT":
      return { ...state, autoEject: action.payload };
    case "SET_SHOW_NOTIFICATION":
      return { ...state, showNotification: action.payload };
    case "RESET_WRITE":
      return { ...state, writePhase: "idle", writeProgress: null, writeError: null };
    default:
      return state;
  }
}

const AppContext = createContext<{
  state: AppState;
  dispatch: Dispatch<AppAction>;
} | null>(null);

function loadTheme(): Theme {
  const stored = localStorage.getItem("gosh-usb-theme");
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

function loadVerifyDefault(): boolean {
  const stored = localStorage.getItem("gosh-usb-verify-default");
  if (stored === "false") {
    return false;
  }
  return true;
}

function loadMode(): AppMode {
  const stored = localStorage.getItem("gosh-usb-mode");
  if (stored === "standard" || stored === "advanced") {
    return stored;
  }
  return "standard";
}

function loadAutoEject(): boolean {
  return localStorage.getItem("gosh-usb-auto-eject") === "true";
}

function loadShowNotification(): boolean {
  const stored = localStorage.getItem("gosh-usb-show-notification");
  return stored !== "false"; // Default to true
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, {
    ...initialState,
    theme: loadTheme(),
    verifyAfterWrite: loadVerifyDefault(),
    mode: loadMode(),
    autoEject: loadAutoEject(),
    showNotification: loadShowNotification(),
  });

  useEffect(() => {
    applyTheme(state.theme);
    localStorage.setItem("gosh-usb-theme", state.theme);
  }, [state.theme]);

  useEffect(() => {
    localStorage.setItem("gosh-usb-verify-default", String(state.verifyAfterWrite));
  }, [state.verifyAfterWrite]);

  useEffect(() => {
    localStorage.setItem("gosh-usb-mode", state.mode);
  }, [state.mode]);

  useEffect(() => {
    localStorage.setItem("gosh-usb-auto-eject", String(state.autoEject));
  }, [state.autoEject]);

  useEffect(() => {
    localStorage.setItem("gosh-usb-show-notification", String(state.showNotification));
  }, [state.showNotification]);

  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useApp must be used within AppProvider");
  }
  return context;
}
