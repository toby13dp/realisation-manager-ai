/// Media service — wraps Tauri media commands.

import { call } from './tauri';
import type { Classification, Media, MediaType } from '@/types';

export interface ListMediaParams {
  projectId?: string;
  classification?: Classification;
  mediaType?: MediaType;
  isPrivate?: boolean;
  isStarred?: boolean;
  isDuplicate?: boolean;
  dateFrom?: string;
  dateTo?: string;
  sourceFolder?: string;
  limit?: number;
  offset?: number;
}

export const mediaService = {
  list: (params: ListMediaParams = {}) =>
    call<Media[]>('list_media', params),

  get: (id: string) => call<Media | null>('get_media', { id }),

  update: (media: Media) => call<void>('update_media', { media }),

  delete: (id: string) => call<void>('delete_media', { id }),

  toggleStar: (id: string, starred: boolean) =>
    call<void>('toggle_star', { id, starred }),

  setClassification: (id: string, classification: Classification, confidence = 1.0) =>
    call<void>('set_classification', { id, classification, confidence }),

  setPrivacy: (id: string, isPrivate: boolean, locked = false) =>
    call<void>('set_privacy', { id, isPrivate, locked }),

  search: (params: {
    query?: string;
    classification?: Classification;
    mediaType?: MediaType;
    projectId?: string;
    isPrivate?: boolean;
    dateFrom?: string;
    dateTo?: string;
    sourceFolder?: string;
    limit?: number;
  }) => call<Media[]>('search_media', params),
};
