/// AI service.

import { call } from './tauri';
import type { AiStatus, AnalysisResult, DetectedProject } from '@/types';

export const aiService = {
  analyzeMedia: (mediaId: string) =>
    call<AnalysisResult>('analyze_media', { mediaId }),

  analyzeBatch: (mediaIds: string[]) =>
    call<AnalysisResult[]>('analyze_batch', { mediaIds }),

  detectProjects: (persist = false) =>
    call<DetectedProject[]>('detect_projects', { persist }),

  status: () => call<AiStatus>('ai_status'),
};
