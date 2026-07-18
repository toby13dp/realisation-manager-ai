/// Settings + folder rules + scanner service.

import { call } from './tauri';
import type { ScanResult } from './types';

export interface Setting {
  key: string;
  value: unknown;
  category: string;
  description?: string | null;
  updatedAt: string;
}

export interface FolderRule {
  id: string;
  folderPath: string;
  classification: 'business' | 'private' | 'exclude';
  recursive: boolean;
  priority: number;
  notes?: string | null;
  createdAt: string;
  updatedAt: string;
}

export const settingsService = {
  get: (key: string) => call<unknown>('get_setting', { key }),
  set: (key: string, value: unknown) => call<void>('set_setting', { key, value }),
  all: () => call<Setting[]>('all_settings'),
  listFolderRules: () => call<FolderRule[]>('list_folder_rules'),
  upsertFolderRule: (rule: FolderRule) => call<void>('upsert_folder_rule', { rule }),
  deleteFolderRule: (id: string) => call<void>('delete_folder_rule', { id }),
};

export const scannerService = {
  scanFolder: (folder: string) => call<ScanResult>('scan_folder', { folder }),
  importFolder: (folder: string) => call<ScanResult>('import_folder', { folder }),
  watchedFolders: () => call<string[]>('watched_folders'),
  addWatchFolder: (folder: string) => call<void>('add_watch_folder', { folder }),
  removeWatchFolder: (folder: string) => call<void>('remove_watch_folder', { folder }),
};

export const jobService = {
  list: (limit = 50) => call<unknown[]>('list_jobs', { limit }),
  cancel: (id: string) => call<boolean>('cancel_job', { id }),
};

export const statsService = {
  dashboard: () =>
    call<{
      totalMedia: number;
      businessCount: number;
      privateCount: number;
      unclassifiedCount: number;
      duplicatesCount: number;
      imagesCount: number;
      videosCount: number;
      projectCount: number;
      approvedProjects: number;
      detectedProjects: number;
      avgQuality: number | null;
      aiStatus: import('@/types').AiStatus;
    }>('dashboard_stats'),
};
