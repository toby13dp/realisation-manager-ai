/// SEO domain types.

export type SeoStatus = 'draft' | 'ready' | 'published' | 'archived';

export interface Seo {
  id: string;
  projectId: string;
  title: string;
  slug: string;
  metaDescription: string;
  keywords: string[];
  canonicalUrl?: string | null;
  ogTitle?: string | null;
  ogDescription?: string | null;
  ogImageMediaId?: string | null;
  bodyHtml?: string | null;
  bodyMarkdown?: string | null;
  altTexts: Record<string, string>;
  schemaOrgJson?: string | null;
  language: string;
  readingTimeMin?: number | null;
  wordCount?: number | null;
  status: SeoStatus;
  generatedBy: string;
  promptTemplate?: string | null;
  createdAt: string;
  updatedAt: string;
}

export const SEO_STATUS_LABELS: Record<SeoStatus, string> = {
  draft: 'Concept',
  ready: 'Klaar',
  published: 'Gepubliceerd',
  archived: 'Gearchiveerd',
};

export const SEO_STATUS_COLORS: Record<SeoStatus, string> = {
  draft: 'bg-gray-100 text-gray-800',
  ready: 'bg-blue-100 text-blue-800',
  published: 'bg-green-100 text-green-800',
  archived: 'bg-gray-200 text-gray-700',
};
