/// SEO service.

import { call } from './tauri';
import type { Seo, SeoStatus } from '@/types';

export interface SeoGenerated {
  seo: Seo;
  warnings: string[];
}

export const seoService = {
  generate: (projectId: string) => call<SeoGenerated>('generate_seo', { projectId }),
  list: (status?: SeoStatus) => call<Seo[]>('list_seo', { status }),
  get: (id: string) => call<Seo | null>('get_seo', { id }),
  update: (seo: Seo) => call<void>('update_seo', { seo }),
  delete: (id: string) => call<void>('delete_seo', { id }),
  exportMarkdown: (id: string) => call<string>('export_seo_markdown', { id }),
};
