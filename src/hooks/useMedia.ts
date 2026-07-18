/// React Query hooks for media.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { mediaService, type ListMediaParams } from '@/services/mediaService';
import type { Classification, Media } from '@/types';
import { toast } from '@/store/toastStore';

export function useMediaList(params: ListMediaParams = {}) {
  return useQuery({
    queryKey: ['media', 'list', params],
    queryFn: () => mediaService.list(params),
  });
}

export function useMedia(id: string | null) {
  return useQuery({
    queryKey: ['media', 'get', id],
    queryFn: () => (id ? mediaService.get(id) : Promise.resolve(null)),
    enabled: !!id,
  });
}

export function useToggleStar() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, starred }: { id: string; starred: boolean }) =>
      mediaService.toggleStar(id, starred),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['media'] }),
    onError: (e) => toast.error(`Ster mislukt: ${e}`),
  });
}

export function useSetClassification() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      classification,
      confidence = 1.0,
    }: {
      id: string;
      classification: Classification;
      confidence?: number;
    }) => mediaService.setClassification(id, classification, confidence),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['media'] }),
    onError: (e) => toast.error(`Classificatie mislukt: ${e}`),
  });
}

export function useSetPrivacy() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      isPrivate,
      locked = false,
    }: {
      id: string;
      isPrivate: boolean;
      locked?: boolean;
    }) => mediaService.setPrivacy(id, isPrivate, locked),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['media'] }),
    onError: (e) => toast.error(`Privacy aanpassen mislukt: ${e}`),
  });
}

export function useDeleteMedia() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => mediaService.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['media'] });
      toast.success('Media verwijderd');
    },
    onError: (e) => toast.error(`Verwijderen mislukt: ${e}`),
  });
}

export function useUpdateMedia() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (media: Media) => mediaService.update(media),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['media'] }),
    onError: (e) => toast.error(`Opslaan mislukt: ${e}`),
  });
}
