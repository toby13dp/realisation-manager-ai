/// Project store — Zustand slice for projects UI.

import { create } from 'zustand';
import type { ProjectStatus, ProjectType } from '@/types';

interface ProjectStoreState {
  filter: {
    status?: ProjectStatus;
    projectType?: ProjectType;
    isPrivate?: boolean;
    query?: string;
  };
  selectedProjectId: string | null;
  setFilter: (filter: Partial<ProjectStoreState['filter']>) => void;
  resetFilter: () => void;
  selectProject: (id: string | null) => void;
}

export const useProjectStore = create<ProjectStoreState>((set) => ({
  filter: {},
  selectedProjectId: null,
  setFilter: (filter) =>
    set((s) => ({ filter: { ...s.filter, ...filter } })),
  resetFilter: () => set({ filter: {} }),
  selectProject: (id) => set({ selectedProjectId: id }),
}));
