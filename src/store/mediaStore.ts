/// Media store — Zustand slice for the currently selected media + filters.

import { create } from 'zustand';
import type { Classification, MediaType, Media } from '@/types';

interface MediaStoreState {
  filter: {
    classification?: Classification;
    mediaType?: MediaType;
    isPrivate?: boolean;
    isStarred?: boolean;
    isDuplicate?: boolean;
    dateFrom?: string;
    dateTo?: string;
    sourceFolder?: string;
    query?: string;
  };
  selectedIds: string[];
  lastResult: Media[];
  setFilter: (filter: Partial<MediaStoreState['filter']>) => void;
  resetFilter: () => void;
  setSelected: (ids: string[]) => void;
  toggleSelected: (id: string) => void;
  clearSelected: () => void;
  setLastResult: (m: Media[]) => void;
}

export const useMediaStore = create<MediaStoreState>((set) => ({
  filter: {},
  selectedIds: [],
  lastResult: [],
  setFilter: (filter) =>
    set((s) => ({ filter: { ...s.filter, ...filter } })),
  resetFilter: () => set({ filter: {} }),
  setSelected: (ids) => set({ selectedIds: ids }),
  toggleSelected: (id) =>
    set((s) => ({
      selectedIds: s.selectedIds.includes(id)
        ? s.selectedIds.filter((x) => x !== id)
        : [...s.selectedIds, id],
    })),
  clearSelected: () => set({ selectedIds: [] }),
  setLastResult: (m) => set({ lastResult: m }),
}));
