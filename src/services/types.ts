/// Shared service types.

export interface ScanResult {
  jobId: string;
  scanned: number;
  inserted: number;
  skipped: number;
  failed: number;
  duplicates: number;
  elapsedMs: number;
  errors: string[];
}

export interface ScanProgress {
  jobId: string;
  current: number;
  total: number;
  failed: number;
  currentFile?: string;
  elapsedMs: number;
}
