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
