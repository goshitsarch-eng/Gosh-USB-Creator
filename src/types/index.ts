export interface BlockDevice {
  path: string;
  name: string;
  size: number;
  size_human: string;
  removable: boolean;
  mount_points: string[];
}

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  size_human: string;
}

export interface ImageValidation {
  is_valid: boolean;
  format: string;
  errors: string[];
  warnings: string[];
}

export interface WriteProgress {
  phase: "writing" | "verifying";
  bytes_written: number;
  total_bytes: number;
  speed_bps: number;
  eta_seconds: number;
}

export interface ChecksumProgress {
  bytes_processed: number;
  total_bytes: number;
}

export type Theme = "light" | "dark" | "system";

export type AppMode = "standard" | "advanced";

export type WritePhase = "idle" | "preparing" | "writing" | "verifying" | "complete" | "error";

export interface AppState {
  selectedFile: FileInfo | null;
  selectedDevice: BlockDevice | null;
  devices: BlockDevice[];
  devicesLoading: boolean;
  calculatedChecksum: string | null;
  checksumAlgorithm: "sha256" | "md5";
  checksumLoading: boolean;
  expectedChecksum: string;
  writePhase: WritePhase;
  writeProgress: WriteProgress | null;
  writeError: string | null;
  verifyAfterWrite: boolean;
  theme: Theme;
  mode: AppMode;
  imageValidation: ImageValidation | null;
  imageValidationLoading: boolean;
  // Post-write actions (Advanced mode)
  autoEject: boolean;
  showNotification: boolean;
}

export type AppAction =
  | { type: "SET_FILE"; payload: FileInfo | null }
  | { type: "SET_DEVICE"; payload: BlockDevice | null }
  | { type: "SET_DEVICES"; payload: BlockDevice[] }
  | { type: "SET_DEVICES_LOADING"; payload: boolean }
  | { type: "SET_CHECKSUM"; payload: string | null }
  | { type: "SET_CHECKSUM_ALGORITHM"; payload: "sha256" | "md5" }
  | { type: "SET_CHECKSUM_LOADING"; payload: boolean }
  | { type: "SET_EXPECTED_CHECKSUM"; payload: string }
  | { type: "SET_WRITE_PHASE"; payload: WritePhase }
  | { type: "SET_WRITE_PROGRESS"; payload: WriteProgress | null }
  | { type: "SET_WRITE_ERROR"; payload: string | null }
  | { type: "SET_VERIFY_AFTER_WRITE"; payload: boolean }
  | { type: "SET_THEME"; payload: Theme }
  | { type: "SET_MODE"; payload: AppMode }
  | { type: "SET_IMAGE_VALIDATION"; payload: ImageValidation | null }
  | { type: "SET_IMAGE_VALIDATION_LOADING"; payload: boolean }
  | { type: "SET_AUTO_EJECT"; payload: boolean }
  | { type: "SET_SHOW_NOTIFICATION"; payload: boolean }
  | { type: "RESET_WRITE" };
