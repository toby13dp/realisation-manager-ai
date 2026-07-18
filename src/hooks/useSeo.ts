/// React Query hooks for SEO.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { seoService } from '@/services/seoService';
import type { Seo, SeoStatus } from '@/types';
import { toast } from '@/store/toastStore';

export function useSeoList(status?: SeoStatus) {
  return useQuery({
    queryKey: ['seo', 'list', status],
    queryFn: () => seoService.list(status),
  });
}

export function useSeo(id: string | null) {
  return useQuery({
    queryKey: ['seo', 'get', id],
    queryFn: () => (id ? seoService.get(id) : Promise.resolve(null)),
    enabled: !!id,
  });
}

export function useGenerateSeo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (projectId: string) => seoService.generate(projectId),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['seo'] }),
    onError: (e) => toast.error(`Genereren mislukt: ${e}`),
  });
}

export function useUpdateSeo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (seo: Seo) => seoService.update(seo),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['seo'] });
      toast.success('SEO opgeslagen');
    },
    onError: (e) => toast.error(`Opslaan mislukt: ${e}`),
  });
}

export function useDeleteSeo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => seoService.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['seo'] });
      toast.success('SEO verwijderd');
    },
    onError: (e) => toast.error(`Verwijderen mislukt: ${e}`),
  });
}
