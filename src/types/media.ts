/// Media domain types — mirror the Rust `Media` model.

export type MediaType = 'image' | 'video' | 'raw' | 'unknown';

export type Classification = 'business' | 'private' | 'unclassified' | 'mixed';

export interface Media {
  id: string;
  filePath: string;
  fileName: string;
  fileExtension: string;
  fileSize: number;
  fileHash: string;
  fullHash?: string | null;
  mimeType?: string | null;
  mediaType: MediaType;
  width?: number | null;
  height?: number | null;
  durationMs?: number | null;
  thumbnailPath?: string | null;
  previewPath?: string | null;
  dateTaken?: string | null;
  dateImported: string;
  sourceFolder?: string | null;
  isPrivate: boolean;
  privacyLocked: boolean;
  classification: Classification;
  classificationConfidence: number;
  projectId?: string | null;
  qualityScore?: number | null;
  isDuplicate: boolean;
  duplicateOf?: string | null;
  isStarred: boolean;
  notes?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface MediaFilter {
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
  orderBy?: string;
}

export interface ExifData {
  id?: number;
  mediaId: string;
  cameraMake?: string | null;
  cameraModel?: string | null;
  lensModel?: string | null;
  software?: string | null;
  iso?: number | null;
  aperture?: number | null;
  shutterSpeed?: string | null;
  focalLength?: number | null;
  gpsLatitude?: number | null;
  gpsLongitude?: number | null;
  gpsAltitude?: number | null;
  gpsTimestamp?: string | null;
  orientation?: number | null;
  colorSpace?: string | null;
  whiteBalance?: string | null;
  exposureBias?: number | null;
  flashFired?: boolean | null;
  originalDateTime?: string | null;
  digitizedDateTime?: string | null;
  rawExifJson?: string | null;
  createdAt?: string;
}
