/// AI domain types.

export type AnalysisType =
  | 'object_detection'
  | 'scene'
  | 'brand'
  | 'ocr'
  | 'embedding'
  | 'quality'
  | 'duplicate'
  | 'classification';

export interface DetectedObject {
  label: string;
  confidence: number;
  bbox: [number, number, number, number]; // x, y, w, h
}

export interface AiAnalysis {
  id: string;
  mediaId: string;
  analysisType: AnalysisType;
  modelName: string;
  modelVersion: string;
  results: unknown;
  confidence: number;
  processingTimeMs?: number | null;
  analyzedAt: string;
}

export interface AnalysisResult {
  mediaId: string;
  objects: DetectedObject[];
  sceneTags: string[];
  brands: string[];
  ocrText?: string;
  qualityScore: number;
  classification: 'business' | 'private' | 'unclassified' | 'mixed';
  classificationConfidence: number;
  embedding: number[];
  processingTimeMs: number;
}

export interface AiStatus {
  objectDetectionLoaded: boolean;
  clipLoaded: boolean;
  ocrAvailable: boolean;
  device: string;
  modelDir: string;
  errors: string[];
}

export interface DetectedProject {
  name: string;
  projectType: import('./project').ProjectType;
  confidence: number;
  mediaIds: string[];
  startDate?: string;
  endDate?: string;
  latitude?: number;
  longitude?: number;
  locationLabel?: string;
  suggestedTags: string[];
}
