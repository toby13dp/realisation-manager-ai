/// Project service.

import { call } from './tauri';
import type { Project, ProjectStatus, ProjectType, ProjectSummary } from '@/types';

export const projectService = {
  list: (params: {
    status?: ProjectStatus;
    projectType?: ProjectType;
    isPrivate?: boolean;
    limit?: number;
    offset?: number;
  } = {}) => call<Project[]>('list_projects', params),

  get: (id: string) => call<Project | null>('get_project', { id }),

  create: (project: Project) => call<void>('create_project', { project }),

  update: (project: Project) => call<void>('update_project', { project }),

  delete: (id: string) => call<void>('delete_project', { id }),

  approve: (id: string) => call<void>('approve_project', { id }),

  merge: (sourceId: string, targetId: string) =>
    call<void>('merge_projects', { sourceId, targetId }),

  split: (sourceId: string, mediaIds: string[], newProjectName: string) =>
    call<Project>('split_project', { sourceId, mediaIds, newProjectName }),

  assignMedia: (mediaIds: string[], projectId: string) =>
    call<void>('assign_media', { mediaIds, projectId }),

  unassignMedia: (mediaIds: string[]) => call<void>('unassign_media', { mediaIds }),

  summary: () => call<ProjectSummary[]>('project_summary'),

  search: (query: string, limit = 50) =>
    call<Project[]>('search_projects', { query, limit }),
};
