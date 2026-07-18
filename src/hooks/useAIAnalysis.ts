/// React Query hooks for AI.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { aiService } from '@/services/aiService';
import { toast } from '@/store/toastStore';

export function useAiStatus() {
  return useQuery({
    queryKey: ['ai', 'status'],
    queryFn: () => aiService.status(),
    refetchInterval: 30000,
  });
}

export function useAnalyzeMedia() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (mediaId: string) => aiService.analyzeMedia(mediaId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['media'] });
      toast.success('Analyse voltooid');
    },
    onError: (e) => toast.error(`Analyse mislukt: ${e}`),
  });
}

export function useAnalyzeBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (mediaIds: string[]) => aiService.analyzeBatch(mediaIds),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['media'] });
      toast.success('Batch-analyse voltooid');
    },
    onError: (e) => toast.error(`Batch-analyse mislukt: ${e}`),
  });
}

export function useDetectProjects() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (persist = false) => aiService.detectProjects(persist),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      toast.success('Projecten gedetecteerd');
    },
    onError: (e) => toast.error(`Detectie mislukt: ${e}`),
  });
}
