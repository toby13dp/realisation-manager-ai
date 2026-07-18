/// File type utilities.

import type { MediaType } from '@/types';

const EXTENSION_MAP: Record<string, MediaType> = {
  jpg: 'image', jpeg: 'image', png: 'image', webp: 'image',
  bmp: 'image', gif: 'image', tiff: 'image', tif: 'image',
  heic: 'image', heif: 'image',
  mov: 'video', mp4: 'video', m4v: 'video', avi: 'video',
  mkv: 'video', webm: 'video',
  nef: 'raw', cr2: 'raw', arw: 'raw', dng: 'raw',
  raf: 'raw', orf: 'raw', rw2: 'raw',
};

export function mediaTypeFromExtension(ext: string): MediaType {
  return EXTENSION_MAP[ext.toLowerCase()] ?? 'unknown';
}

export function isSupportedExtension(ext: string): boolean {
  return ext.toLowerCase() in EXTENSION_MAP;
}

export const SUPPORTED_EXTENSIONS = Object.keys(EXTENSION_MAP);
