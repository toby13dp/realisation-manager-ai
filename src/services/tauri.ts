/// Thin wrapper around @tauri-apps/api invoke + event listen.
/// All other services use this so we can swap out the transport easily.

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export async function call<T = unknown>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  return invoke<T>(command, args);
}

export function onEvent<T = unknown>(
  name: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(name, (event) => handler(event.payload));
}

export function assetUrl(path: string): string {
  // Convert a local filesystem path to a tauri:// asset URL.
  if (!path) return '';
  // The tauri.conf.json assetProtocol scope allows these.
  // Vite/Tauri dev: https://asset.localhost/...
  // Build: same protocol.
  const encoded = encodeURIComponent(path);
  return `https://asset.localhost/${encoded}`;
}
