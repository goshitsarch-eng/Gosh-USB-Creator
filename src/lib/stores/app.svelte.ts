import type { BlockDevice, FileInfo, ImageValidation, WriteProgress, Theme, AppMode, WritePhase } from "../types";

function loadTheme(): Theme {
  if (typeof localStorage === "undefined") return "system";
  const stored = localStorage.getItem("gosh-usb-theme");
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

function loadVerifyDefault(): boolean {
  if (typeof localStorage === "undefined") return true;
  const stored = localStorage.getItem("gosh-usb-verify-default");
  if (stored === "false") {
    return false;
  }
  return true;
}

function loadMode(): AppMode {
  if (typeof localStorage === "undefined") return "standard";
  const stored = localStorage.getItem("gosh-usb-mode");
  if (stored === "standard" || stored === "advanced") {
    return stored;
  }
  return "standard";
}

function loadAutoEject(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem("gosh-usb-auto-eject") === "true";
}

function loadShowNotification(): boolean {
  if (typeof localStorage === "undefined") return true;
  const stored = localStorage.getItem("gosh-usb-show-notification");
  return stored !== "false";
}

function applyTheme(theme: Theme) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

class AppState {
  // File and device selection
  selectedFile = $state<FileInfo | null>(null);
  selectedDevice = $state<BlockDevice | null>(null);
  devices = $state<BlockDevice[]>([]);
  devicesLoading = $state(false);

  // Checksum
  calculatedChecksum = $state<string | null>(null);
  checksumAlgorithm = $state<"sha256" | "md5">("sha256");
  checksumLoading = $state(false);
  expectedChecksum = $state("");

  // Write operation
  writePhase = $state<WritePhase>("idle");
  writeProgress = $state<WriteProgress | null>(null);
  writeError = $state<string | null>(null);

  // Settings
  verifyAfterWrite = $state(loadVerifyDefault());
  theme = $state<Theme>(loadTheme());
  mode = $state<AppMode>(loadMode());

  // Image validation
  imageValidation = $state<ImageValidation | null>(null);
  imageValidationLoading = $state(false);

  // Post-write actions (Advanced mode)
  autoEject = $state(loadAutoEject());
  showNotification = $state(loadShowNotification());

  // Derived states
  isWriting = $derived(
    this.writePhase === "preparing" ||
    this.writePhase === "writing" ||
    this.writePhase === "verifying"
  );

  canWrite = $derived(
    this.selectedFile !== null &&
    this.selectedDevice !== null &&
    this.writePhase === "idle"
  );

  constructor() {
    // Apply initial theme
    applyTheme(this.theme);
  }

  // Actions
  setFile(file: FileInfo | null) {
    this.selectedFile = file;
    this.calculatedChecksum = null;
    this.imageValidation = null;
  }

  setDevice(device: BlockDevice | null) {
    this.selectedDevice = device;
  }

  setDevices(devices: BlockDevice[]) {
    this.devices = devices;
  }

  setDevicesLoading(loading: boolean) {
    this.devicesLoading = loading;
  }

  setChecksum(checksum: string | null) {
    this.calculatedChecksum = checksum;
  }

  setChecksumAlgorithm(algorithm: "sha256" | "md5") {
    this.checksumAlgorithm = algorithm;
    this.calculatedChecksum = null;
  }

  setChecksumLoading(loading: boolean) {
    this.checksumLoading = loading;
  }

  setExpectedChecksum(checksum: string) {
    this.expectedChecksum = checksum;
  }

  setWritePhase(phase: WritePhase) {
    this.writePhase = phase;
  }

  setWriteProgress(progress: WriteProgress | null) {
    this.writeProgress = progress;
  }

  setWriteError(error: string | null) {
    this.writeError = error;
    if (error) {
      this.writePhase = "error";
    }
  }

  setVerifyAfterWrite(verify: boolean) {
    this.verifyAfterWrite = verify;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("gosh-usb-verify-default", String(verify));
    }
  }

  setTheme(theme: Theme) {
    this.theme = theme;
    applyTheme(theme);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("gosh-usb-theme", theme);
    }
  }

  setMode(mode: AppMode) {
    this.mode = mode;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("gosh-usb-mode", mode);
    }
  }

  setImageValidation(validation: ImageValidation | null) {
    this.imageValidation = validation;
  }

  setImageValidationLoading(loading: boolean) {
    this.imageValidationLoading = loading;
  }

  setAutoEject(autoEject: boolean) {
    this.autoEject = autoEject;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("gosh-usb-auto-eject", String(autoEject));
    }
  }

  setShowNotification(show: boolean) {
    this.showNotification = show;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("gosh-usb-show-notification", String(show));
    }
  }

  resetWrite() {
    this.writePhase = "idle";
    this.writeProgress = null;
    this.writeError = null;
  }
}

export const appState = new AppState();
