/// Settings store + privacy mode + theme.

import { create } from 'zustand';

interface SettingsStoreState {
  settings: Record<string, unknown>;
  privacyMode: boolean;
  theme: 'light' | 'dark' | 'system';
  loaded: boolean;
  setAll: (settings: Record<string, unknown>) => void;
  set: (key: string, value: unknown) => void;
  togglePrivacyMode: () => void;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

export const useSettingsStore = create<SettingsStoreState>((set, get) => ({
  settings: {},
  privacyMode: false,
  theme: 'light',
  loaded: false,
  setAll: (settings) =>
    set({
      settings,
      privacyMode: !!settings['app.privacy_mode'],
      theme: (settings['app.theme'] as 'light' | 'dark' | 'system') ?? 'light',
      loaded: true,
    }),
  set: (key, value) =>
    set((s) => ({ settings: { ...s.settings, [key]: value } })),
  togglePrivacyMode: () =>
    set((s) => {
      const next = !s.privacyMode;
      return { privacyMode: next, settings: { ...s.settings, 'app.privacy_mode': next } };
    }),
  setTheme: (theme) => set({ theme }),
}));

export const getSetting = <T = unknown>(state: SettingsStoreState, key: string, fallback: T): T => {
  const v = state.settings[key];
  return (v as T) ?? fallback;
};
